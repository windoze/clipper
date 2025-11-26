use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use clipper_client::{Clip, ClipperClient, SearchFilters};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak as ArcWeak};

slint::include_modules!();

const PAGE_SIZE: usize = 200;
const FAVORITE_TAG: &str = "$favorite";

fn main() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("Failed to start Tokio runtime")?;
    let base_url = env::var("CLIPPER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let client = ClipperClient::new(base_url);

    let app = App::new().map_err(|e| anyhow!("Failed to initialize UI: {e}"))?;
    let controller = AppController::new(client, runtime.handle().clone(), app.as_weak());

    {
        let controller = controller.clone();
        app.on_search_text_changed(move |text| {
            controller.update_search(text.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_start_date_changed(move |text| {
            controller.update_start_date(text.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_end_date_changed(move |text| {
            controller.update_end_date(text.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_favorites_only_changed(move |value| {
            controller.update_favorites_only(value);
        });
    }

    {
        let controller = controller.clone();
        app.on_toggle_favorite(move |id| {
            controller.toggle_favorite(id.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_refresh_request(move || {
            controller.load_clips();
        });
    }

    controller.load_clips();

    app.run().map_err(|e| anyhow!("UI error: {e}"))?;
    drop(runtime);

    Ok(())
}

#[derive(Clone, Default)]
struct FilterState {
    search_text: String,
    start_date_input: String,
    end_date_input: String,
    favorites_only: bool,
}

impl FilterState {
    fn prepare(&self) -> Result<PreparedFilters, String> {
        let mut filters = SearchFilters::new();

        if let Some(start) = parse_date(&self.start_date_input, "start")? {
            filters.start_date = Some(start);
        }

        if let Some(end) = parse_date(&self.end_date_input, "end")? {
            filters.end_date = Some(end);
        }

        if self.favorites_only {
            filters.tags = Some(vec![FAVORITE_TAG.to_string()]);
        }

        Ok(PreparedFilters {
            query: self.search_text.trim().to_string(),
            search_filters: filters,
        })
    }
}

struct PreparedFilters {
    query: String,
    search_filters: SearchFilters,
}

struct AppController {
    client: ClipperClient,
    runtime: tokio::runtime::Handle,
    ui: slint::Weak<App>,
    filters: Mutex<FilterState>,
    cache: Arc<Mutex<HashMap<String, Clip>>>,
}

impl AppController {
    fn new(
        client: ClipperClient,
        runtime: tokio::runtime::Handle,
        ui: slint::Weak<App>,
    ) -> Arc<Self> {
        Arc::new(Self {
            client,
            runtime,
            ui,
            filters: Mutex::new(FilterState::default()),
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn load_clips(self: &Arc<Self>) {
        let prepared = {
            let filters = self.filters.lock().unwrap().clone();
            filters.prepare()
        };

        let prepared = match prepared {
            Ok(p) => p,
            Err(err) => {
                update_status(&self.ui, SharedString::from(err));
                return;
            }
        };

        update_status(&self.ui, SharedString::from("Loading clips..."));

        let client = self.client.clone();
        let ui = self.ui.clone();
        let cache = self.cache.clone();
        let query = prepared.query;
        let search_filters = prepared.search_filters;

        self.runtime.spawn(async move {
            let response = if query.is_empty() {
                client.list_clips(search_filters, 1, PAGE_SIZE).await
            } else {
                client
                    .search_clips(&query, search_filters, 1, PAGE_SIZE)
                    .await
            };

            match response {
                Ok(result) => {
                    {
                        let mut cache_guard = cache.lock().unwrap();
                        cache_guard.clear();
                        for clip in &result.items {
                            cache_guard.insert(clip.id.clone(), clip.clone());
                        }
                    }
                    let entries: Vec<ClipEntryData> =
                        result.items.iter().map(clip_to_ui_entry).collect();
                    update_clip_list(&ui, entries);
                    update_status(
                        &ui,
                        SharedString::from(format!("Showing {} clip(s)", result.items.len())),
                    );
                }
                Err(err) => {
                    update_status(
                        &ui,
                        SharedString::from(format!("Failed to load clips: {err}")),
                    );
                }
            }
        });
    }

    fn update_search(self: &Arc<Self>, text: String) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.search_text = text;
        }
        self.load_clips();
    }

    fn update_start_date(self: &Arc<Self>, text: String) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.start_date_input = text;
        }
        self.load_clips();
    }

    fn update_end_date(self: &Arc<Self>, text: String) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.end_date_input = text;
        }
        self.load_clips();
    }

    fn update_favorites_only(self: &Arc<Self>, value: bool) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.favorites_only = value;
        }
        self.load_clips();
    }

    fn toggle_favorite(self: &Arc<Self>, id: String) {
        let clip = {
            let cache = self.cache.lock().unwrap();
            cache.get(&id).cloned()
        };

        let Some(clip) = clip else {
            update_status(
                &self.ui,
                SharedString::from("Clip not found in current list"),
            );
            return;
        };

        let mut tags = clip.tags.clone();
        let is_favorite = tags.iter().any(|tag| tag == FAVORITE_TAG);
        if is_favorite {
            tags.retain(|tag| tag != FAVORITE_TAG);
        } else {
            tags.push(FAVORITE_TAG.to_string());
        }

        update_status(&self.ui, SharedString::from("Updating favorite..."));

        let client = self.client.clone();
        let ui = self.ui.clone();
        let cache = self.cache.clone();
        let weak_self: ArcWeak<Self> = Arc::downgrade(self);

        self.runtime.spawn(async move {
            match client.update_clip(&clip.id, Some(tags), None).await {
                Ok(updated) => {
                    {
                        let mut cache_guard = cache.lock().unwrap();
                        cache_guard.insert(updated.id.clone(), updated.clone());
                    }

                    let status = if updated.tags.iter().any(|tag| tag == FAVORITE_TAG) {
                        "Marked as favorite"
                    } else {
                        "Removed favorite"
                    };
                    update_status(&ui, SharedString::from(status));

                    if let Some(controller) = weak_self.upgrade() {
                        controller.load_clips();
                    }
                }
                Err(err) => {
                    update_status(
                        &ui,
                        SharedString::from(format!("Failed to update favorite: {err}")),
                    );
                }
            }
        });
    }
}

fn parse_date(input: &str, label: &str) -> Result<Option<DateTime<Utc>>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|_| format!("Invalid {label} date. Use YYYY-MM-DD"))?;

    let datetime = if label == "end" {
        date.and_hms_opt(23, 59, 59)
    } else {
        date.and_hms_opt(0, 0, 0)
    }
    .ok_or_else(|| format!("Invalid {label} date"))?;

    Ok(Some(Utc.from_utc_datetime(&datetime)))
}

fn clip_to_ui_entry(clip: &Clip) -> ClipEntryData {
    ClipEntryData {
        id: clip.id.clone().into(),
        content: clip.content.clone().into(),
        created_at: format_timestamp(&clip.created_at).into(),
        tags: clip.tags.join(", ").into(),
        favorite: clip.tags.iter().any(|tag| tag == FAVORITE_TAG),
    }
}

fn format_timestamp(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn update_status(ui: &slint::Weak<App>, text: SharedString) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = ui.upgrade() {
            app.set_status_text(text.clone());
        }
    });
}

fn update_clip_list(ui: &slint::Weak<App>, clips: Vec<ClipEntryData>) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = ui.upgrade() {
            let model: ModelRc<ClipEntryData> = Rc::new(VecModel::from(clips)).into();
            app.set_clips(model.clone());
        }
    });
}

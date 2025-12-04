// Shared Clip Page JavaScript
// Data is passed from the HTML template via global variables:
// - originalContent: The original clip content for copying
// - expiresAtIso: ISO timestamp of expiration (or null if never expires)

// Translations
const translations = {
    en: {
        pageTitle: '📎 Shared Clip',
        copyToClipboard: 'Copy to Clipboard',
        copied: 'Copied!',
        downloadFile: 'Download File',
        expires: 'Expires',
        never: 'never',
        expired: 'expired',
        inLessThanAMinute: 'in less than a minute',
        inMinutes: 'in {n} minute',
        inMinutesPlural: 'in {n} minutes',
        inHours: 'in {n} hour',
        inHoursPlural: 'in {n} hours',
        inDays: 'in {n} day',
        inDaysPlural: 'in {n} days',
        inMonths: 'in {n} month',
        inMonthsPlural: 'in {n} months',
        copyFailed: 'Failed to copy'
    },
    zh: {
        pageTitle: '📎 分享的剪贴',
        copyToClipboard: '复制到剪贴板',
        copied: '已复制！',
        downloadFile: '下载文件',
        expires: '过期时间',
        never: '永不过期',
        expired: '已过期',
        inLessThanAMinute: '不到一分钟后',
        inMinutes: '{n} 分钟后',
        inMinutesPlural: '{n} 分钟后',
        inHours: '{n} 小时后',
        inHoursPlural: '{n} 小时后',
        inDays: '{n} 天后',
        inDaysPlural: '{n} 天后',
        inMonths: '{n} 个月后',
        inMonthsPlural: '{n} 个月后',
        copyFailed: '复制失败'
    }
};

// Detect language from browser
function detectLanguage() {
    const browserLang = navigator.language || navigator.userLanguage || 'en';
    // Check if Chinese (zh, zh-CN, zh-TW, zh-HK, etc.)
    if (browserLang.startsWith('zh')) {
        return 'zh';
    }
    return 'en';
}

const currentLang = detectLanguage();
const t = translations[currentLang];

// Update HTML lang attribute
document.documentElement.lang = currentLang === 'zh' ? 'zh-CN' : 'en';

// Update page title
document.title = 'Clipper - ' + (currentLang === 'zh' ? '分享的剪贴' : 'Shared Clip');

// Update static text elements
document.getElementById('page-title').textContent = t.pageTitle;
document.getElementById('copy-btn').textContent = t.copyToClipboard;

// Update download button if present
const downloadBtn = document.getElementById('download-btn');
if (downloadBtn) {
    downloadBtn.textContent = t.downloadFile;
}

// Update expiration text for "never" case
const noExpirySpan = document.querySelector('.no-expiry');
if (noExpirySpan) {
    noExpirySpan.textContent = t.never;
    document.getElementById('expiration-info').innerHTML = t.expires + ': <span class="no-expiry">' + t.never + '</span>';
}

function copyToClipboard() {
    navigator.clipboard.writeText(originalContent).then(function() {
        const btn = document.getElementById('copy-btn');
        btn.textContent = t.copied;
        btn.classList.add('btn-success');
        setTimeout(function() {
            btn.textContent = t.copyToClipboard;
            btn.classList.remove('btn-success');
        }, 2000);
    }).catch(function(err) {
        alert(t.copyFailed + ': ' + err);
    });
}

// Format relative time for future (e.g., "in 2 hours", "in 3 days")
function formatFutureRelativeTime(date) {
    const now = new Date();
    const diffMs = date - now;
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMs < 0) {
        return t.expired;
    } else if (diffMins < 1) {
        return t.inLessThanAMinute;
    } else if (diffMins < 60) {
        const template = diffMins === 1 ? t.inMinutes : t.inMinutesPlural;
        return template.replace('{n}', diffMins);
    } else if (diffHours < 24) {
        const template = diffHours === 1 ? t.inHours : t.inHoursPlural;
        return template.replace('{n}', diffHours);
    } else if (diffDays < 30) {
        const template = diffDays === 1 ? t.inDays : t.inDaysPlural;
        return template.replace('{n}', diffDays);
    } else {
        const diffMonths = Math.floor(diffDays / 30);
        const template = diffMonths === 1 ? t.inMonths : t.inMonthsPlural;
        return template.replace('{n}', diffMonths);
    }
}

// Get CSS class based on time until expiration
function getExpirationClass(date) {
    const now = new Date();
    const diffMs = date - now;
    const diffHours = diffMs / (1000 * 60 * 60);

    if (diffHours < 0) {
        return 'expires-warning';
    } else if (diffHours < 1) {
        return 'expires-warning';
    } else if (diffHours < 24) {
        return 'expires-soon';
    }
    return '';
}

// Update expiration display with local time
function updateExpirationDisplay() {
    if (typeof expiresAtIso === 'undefined' || !expiresAtIso) return;

    const expiresAt = new Date(expiresAtIso);
    const localTimeStr = expiresAt.toLocaleString();
    const relativeStr = formatFutureRelativeTime(expiresAt);
    const expirationClass = getExpirationClass(expiresAt);

    const container = document.getElementById('expiration-info');
    container.innerHTML = t.expires + ': <span class="time-relative ' + expirationClass + '" title="' + localTimeStr + '">' + relativeStr + '</span>';
}

// Update expiration display on page load
if (typeof expiresAtIso !== 'undefined' && expiresAtIso) {
    updateExpirationDisplay();
    // Update every minute
    setInterval(updateExpirationDisplay, 60000);
}

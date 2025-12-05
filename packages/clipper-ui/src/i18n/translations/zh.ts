export const zh = {
  // App
  "app.title": "Clipper",
  "app.clips_count": "{count} 条剪贴",

  // Common
  "common.save": "保存",
  "common.cancel": "取消",
  "common.delete": "删除",
  "common.close": "关闭",
  "common.loading": "加载中...",
  "common.error": "错误",
  "common.saving": "保存中...",
  "common.deleting": "删除中...",

  // Clip Entry
  "clip.copy": "点击复制",
  "clip.share": "分享剪贴",
  "clip.edit": "编辑剪贴",
  "clip.delete": "删除剪贴",
  "clip.favorite.add": "添加到收藏",
  "clip.favorite.remove": "从收藏中移除",
  "clip.download": "点击下载",
  "clip.delete_confirm": "确定要删除这条剪贴吗？",
  "clip.expand": "展开",
  "clip.collapse": "收起",
  "clip.selectLanguage": "选择语法高亮语言",

  // Date Tag
  "dateTag.setStartDate": "从此日期开始筛选",
  "dateTag.setEndDate": "筛选到此日期",

  // Clip List
  "clipList.loading": "加载剪贴中...",
  "clipList.error": "错误：{error}",
  "clipList.empty": "没有找到剪贴",
  "clipList.loadingMore": "加载更多...",
  "clipList.noMore": "没有更多了",

  // Search & Filters
  "search.label": "搜索",
  "search.placeholder": "搜索剪贴...",
  "search.placeholderWithTags": "在筛选的标签中搜索...",
  "filter.from": "从：",
  "filter.to": "至：",
  "filter.startDate": "开始日期",
  "filter.endDate": "结束日期",
  "filter.favorites": "仅显示收藏",
  "filter.removeTag": "移除标签筛选",
  "filter.clearAll": "清除所有筛选",
  "filter.clickToFilter": "点击按此标签筛选",

  // Settings (base settings shared by all apps)
  "settings.title": "设置",
  "settings.appearance": "外观",
  "settings.theme": "主题",
  "settings.theme.light": "浅色",
  "settings.theme.dark": "深色",
  "settings.theme.auto": "自动",
  "settings.theme.hint": "选择您偏好的颜色主题。自动模式跟随系统设置。",
  "settings.syntaxTheme": "代码语法主题",
  "settings.syntaxTheme.github": "GitHub",
  "settings.syntaxTheme.monokai": "Monokai",
  "settings.syntaxTheme.dracula": "Dracula",
  "settings.syntaxTheme.nord": "Nord",
  "settings.syntaxTheme.hint": "选择代码片段的语法高亮颜色主题。",
  "settings.language": "语言",
  "settings.language.hint": "选择您偏好的语言。",
  "settings.language.en": "英语",
  "settings.language.zh": "简体中文",

  // Edit Clip Dialog
  "editClip.title": "编辑剪贴",
  "editClip.tags": "标签",
  "editClip.tags.placeholder": "添加标签...",
  "editClip.tags.hint": "按 Enter 添加标签，按 Backspace 删除最后一个。",
  "editClip.notes": "备注",
  "editClip.notes.placeholder": "添加关于这条剪贴的备注...",
  "editClip.saveError": "保存失败：{error}",

  // Share Dialog
  "share.title": "分享剪贴",
  "share.generating": "正在生成分享链接...",
  "share.copy": "复制",
  "share.copied": "已复制！",
  "share.hint": "任何拥有此链接的人都可以查看这条剪贴。",
  "share.error": "生成分享链接失败",
  "share.notAvailable": "分享功能不可用",

  // Image Popup
  "imagePopup.download": "下载",

  // Tooltips
  "tooltip.settings": "设置",
  "tooltip.refresh": "刷新",
  "tooltip.copy": "复制到剪贴板",
  "tooltip.sendClipboard": "发送剪贴板内容",

  // File Drop
  "fileDrop.hint": "拖拽文件到此处上传",
  "fileDrop.uploading": "上传中...",

  // Status Indicator
  "status.wsConnected": "实时同步",
  "status.wsDisconnected": "已断开",
  "status.wsUnavailable": "需要 HTTPS",

  // Toast Messages
  "toast.clipCopied": "已复制到剪贴板",
  "toast.clipSaved": "剪贴已保存",
  "toast.clipDeleted": "剪贴已删除",
  "toast.copyFailed": "复制到剪贴板失败",
  "toast.serverError": "服务器连接错误",
  "toast.newClip": "新剪贴已添加",
  "toast.clipUpdated": "剪贴已更新",
  "toast.clipsCleanedUp": "已清理 {count} 条旧剪贴",
  "toast.wsConnected": "实时同步已连接",
  "toast.wsDisconnected": "实时同步已断开",
  "toast.fileUploaded": "文件已上传",
  "toast.uploadFailed": "文件上传失败",
  "toast.clipboardSent": "剪贴板内容已发送",
  "toast.clipboardEmpty": "剪贴板为空",
  "toast.clipboardReadFailed": "读取剪贴板失败",
  "toast.wsAuthFailed": "WebSocket 身份验证失败",

  // Connection Error
  "connectionError.title": "无法连接",
  "connectionError.description": "无法连接到 Clipper 服务器。可能的原因：",
  "connectionError.reason.serverDown": "服务器未运行或无法访问",
  "connectionError.reason.networkIssue": "网络连接问题",
  "connectionError.reason.wrongUrl": "设置中的服务器地址不正确",
  "connectionError.retry": "重试",
  "connectionError.openSettings": "打开设置",
  "connectionError.hint": "如果问题持续存在，请检查服务器是否正在运行并验证您的网络连接。",

  // Authentication
  "auth.title": "需要身份验证",
  "auth.description": "此服务器需要身份验证。请输入您的访问令牌。",
  "auth.tokenLabel": "访问令牌",
  "auth.tokenPlaceholder": "输入您的访问令牌...",
  "auth.login": "登录",
  "auth.logout": "退出登录",
  "auth.loggingIn": "登录中...",
  "auth.error": "身份验证失败。请检查您的令牌并重试。",
  "auth.invalidToken": "无效的令牌",
  "auth.sessionExpired": "会话已过期。请重新登录。",

  // Certificate Trust Dialog
  "certificate.title": "不受信任的证书",
  "certificate.warning":
    "服务器使用的是自签名或不受信任的证书。只有在您信任此服务器并已验证证书指纹的情况下才能继续。",
  "certificate.host": "服务器",
  "certificate.fingerprint": "SHA-256 指纹",
  "certificate.hint":
    "如果您信任此证书，它将被保存，以后连接此服务器时不会再提示。",
  "certificate.trust": "信任证书",
  "certificate.trusting": "信任中...",
  "toast.certificateTrusted": "已信任 {host} 的证书",
  "toast.certificateError": "验证证书失败",
};

//! Loopian で使うフォント
//!
//! nannou 0.20 ではテキストのフォントをファミリー名で指定する。
//! 各ファイルは assets/fonts にあり、起動時に読み込んで登録する。

/// フォントのファミリー名
pub type FontName = &'static str;

/// JetBrains Mono ExtraLight（通常テキスト）
pub const FONT_NRM: FontName = "JetBrains Mono ExtraLight";
/// JetBrains Mono ExtraLight Italic（返答テキスト）
pub const FONT_ITALIC: FontName = "JetBrains Mono ExtraLight Italic";
/// JetBrains Mono SemiBold（インジケータ・入力欄）
pub const FONT_BOLD: FontName = "JetBrains Mono SemiBold";
/// JetBrains Mono ExtraBold（タイトル）
pub const FONT_TITLE: FontName = "JetBrains Mono ExtraBold";
/// Newsreader 16pt（ロゴ）
pub const FONT_SERIF: FontName = "Newsreader 16pt";

/// assets/fonts 内のファイル名
pub const FONT_FILES: [&str; 5] = [
    "JetBrainsMono-ExtraLight-Loopian.ttf",
    "JetBrainsMono-ExtraLightItalic-Loopian.ttf",
    "JetBrainsMono-SemiBold-Loopian.ttf",
    "JetBrainsMono-ExtraBold-Loopian.ttf",
    "Newsreader16pt-Regular.ttf",
];

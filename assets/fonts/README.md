# Bundled emoji fallback

`NotoColorEmoji.ttf` comes from Google Fonts' Noto Emoji project:

- Source revision: `99701e07d3d1fe1bdbcdb14964547f0fd592966f`
- File: https://github.com/googlefonts/noto-emoji/blob/99701e07d3d1fe1bdbcdb14964547f0fd592966f/2D/fonts/NotoColorEmoji.ttf
- License: SIL Open Font License 1.1 (see `NotoColorEmoji-LICENSE.txt`).

This fallback supplies newer emoji that the platform's preferred emoji font
cannot draw. Keep coverage aligned with the emoji picker; the
`every_picker_emoji_has_a_bundled_picture` regression test checks its entries.

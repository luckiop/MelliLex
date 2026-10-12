# Changelog

All notable changes to MelliLex will be documented here.

## [0.12.1] — 2026-09-04

- Related words no longer block the next lookup; section 3 can time out on its own
- Back arrow returns to the empty home screen
- Search box stays one ellipsis line while a long query is showing
- Remarks are longer reader notes and must not mention translation
- Home always shows the beta version, even if the OS API is slow

## [0.12.0] — 2026-09-04

Internal beta (`0.x`).

- Same search box can translate: long text, mixed Chinese–English, or sentence punctuation
- Translation result shows source, Chinese, and an optional remark
- One adaptive prompt (technical stays precise; literary can paraphrase)
- Query cap for translation is 8000 characters
- NSIS setup can update files in an existing install folder
- First launch of a new version shows a short what’s-new note

## [0.11.0] — 2026-09-03

Internal beta (`0.x`). Version is shown on the home screen.

- Split DeepSeek into its own provider; OpenAI again uses api.openai.com
- Repair common lookup JSON mistakes (missing/trailing commas)
- English-only related-word chips; Chinese glosses no longer open zh-en
- Variable meaning count ranked by usefulness; display cap is a UI setting
- History page GitHub-style lookup heatmap

## [0.10.0] — 2026-05-22


- Updated OpenAI and Gemini models
- Fix calibre text detection
- Fix an issue where window is not visible during a lookup

## [0.9.0] — 2026-05-04

### Capture & global lookup
- Replaced the keyboard shortcut with **Ctrl + Right-click** as the only global lookup trigger
- Expanded OCR capture detection to cover PDF readers and Kindle Cloud Reader
- Word-walk fallback for Chromium-based windows; strip stray U+FFFC object-replacement characters from captured text
- Removed the unreliable clipboard capture strategy

### AI providers
- Refreshed model lists: Claude Opus 4.7, Gemini 3.1 Pro Preview / Flash-Lite Preview, GPT-5.3 / GPT-5.4
- GPT-5 mini is now the default OpenAI model (GPT-5 Nano removed)
- Anthropic key validation now uses `GET /v1/models`
- Provider error messages from the upstream API are surfaced to the UI instead of a generic `400 Bad Request`

## [0.8.0] — 2026-03-12

### Initial public release

**Core lookup**
- Word and phrase lookup powered by OpenAI, Anthropic, Google Gemini, or local Ollama
- Six result sections: meanings, usage patterns, related words, formality, domain context, common mistakes
- Global keyboard shortcut to look up selected text from any app
- OCR-based capture for non-selectable text on screen

**Explore mode**
- Practice exercises with regeneration
- Common mistakes analysis
- Formality and domain exploration cards

**Quality of life**
- SymSpell spell-check with correction suggestions
- Search history with re-lookup
- Export to Markdown, Capacities.app, or plain text
- 11 UI languages
- Auto-update support via GitHub Releases
- Windows NSIS installer

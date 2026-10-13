/**
 * 0.13.0 — natural English voice, voice controls, and a storage view.
 *
 * The popup copy is `whatsNew.items` in every locale. The public list is
 * CHANGELOG.md. This file is the implementation map for later versions.
 *
 * Voice
 * - Words and phrases only. Kokoro runs in `mellilex_kokoro.dll`, loaded after
 *   the sherpa libraries exist. The main executable does not link them, so the
 *   app still starts when they are missing.
 * - `build.rs` builds that plugin. Only a release build copies it into
 *   `src-tauri/resources`, which is what the installer ships. A debug build
 *   must not replace that file.
 * - The first pronounce checks the plugin, the speech libraries, and the voice
 *   model, then offers to install whatever is missing. Declining keeps the
 *   system voice (`localStorage` key `mellilex.pronunciationEngine`).
 * - Load uses `LOAD_WITH_ALTERED_SEARCH_PATH` from the directory that holds
 *   the sherpa DLLs. A plugin already sitting there is replaced when the
 *   bundled file is newer or a different size. Leaving the old plugin in place
 *   crashed: the new call passes speed and speaker, and the old export read
 *   those numbers as pointers (0xc0000005).
 * - Speed and speaker go to sherpa `GenerationConfig`. There is no volume
 *   control in the model, and an HTML audio element cannot play louder than
 *   1, so loudness is a PCM gain applied before the WAV is written. Defaults
 *   are gain 1.7, speed 0.9, and speaker 3 (`af_sarah`) instead of the breathy
 *   speaker 2 (`af_nicole`).
 * - Recordings live in app-data `voice-cache`. The file name is the normalized
 *   text plus the quantized volume, speed, and speaker. A hit returns the wav
 *   and does not call the model. Moving a slider creates a new file and does
 *   not delete the others.
 *
 * Storage
 * - Settings → Storage measures lookup text, voice recordings, and logs, each
 *   with its own 7/30/90-day delete. The voice model, speech libraries, and
 *   the database file are shown and are not part of that delete.
 * - Lookup rows live inside `mellilex.db`. After they are deleted the file is
 *   vacuumed so the reported size can shrink. A log that is still open is
 *   emptied when it cannot be removed.
 */
export const RELEASE_VERSION = '0.13.0';

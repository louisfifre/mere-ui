# Fonts

Portable font data, shared by every backend (the Windows one loads it today, a Linux one will read the same files).

| File | Content |
|------|---------|
| `IBMPlexMono-Regular-Latin1.woff2` | IBM Plex Mono, weight 400, Latin-1 subset |
| `IBMPlexMono-SemiBold-Latin1.woff2` | IBM Plex Mono, weight 600, Latin-1 subset |
| `IBMPlexMono-Regular-Pi.woff2` | IBM Plex Mono, weight 400, arrows and signs (U+2190-2199, U+2248, U+2260, U+2264-2265, U+2713) |

Origin: IBM Plex Mono release `@ibm/plex-mono@2.5.0`, as embedded (base64 woff2) in the `@font-face` rules of the
world preview's `page/preview.html`, decoded unchanged. The page and the native interface therefore draw the same glyphs.

Licence: SIL Open Font License 1.1, copyright IBM Corp. (Reserved Font Name "Plex"); see `OFL.txt`.
The text there was written from the standard licence text, not copied from a file on this machine (none was found).
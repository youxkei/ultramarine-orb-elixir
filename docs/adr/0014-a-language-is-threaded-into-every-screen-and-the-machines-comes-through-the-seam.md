# 14. A language is threaded into every screen, and the machine's own comes through the seam

**Status:** accepted and built. `orb_config::Language` is the two languages orb has words for;
`orb.yaml`'s `language` is one of them or `auto`; `orb_api::locale::ui_language` is
`GetUserDefaultUILanguage` behind the seam, and `Language::of_the_machine` is what reads it. Each of the
three screens holds the language it was made with — `ModeMenu`, `RetryMenu`, `ResumeMenu` and
`resume_ui::Mark` — and every string of them is a `match` over it, beside the screen itself. The launcher
does the same for its dialog and for the two refusals somebody playing can meet. `orb-e2e`'s
`the_screens_in_english` declares a machine whose windows are English and reads the screens back.

## Context

**orb's screens were Japanese, and the game is not the reason they had to stay so.** 紅魔郷 is a Japanese
game whose text comes out of its own data files, and orb draws none of that: what orb draws is its own
three questions and its own settings dialog. Somebody playing the game with a translation patch over it
reads English, and every word orb put on the screen was a word they could not read.

**Which language that person reads is a thing Windows already knows.**
`GetUserDefaultUILanguage` answers the language Windows shows its own windows in, and it is not
`GetUserDefaultLocaleName`: that one carries the regional formats, which are set apart from the language
and are set to Japanese on machines whose windows are English.

**Both halves of orb need the answer, and neither may ask Windows directly.** `orb-core` is checked for a
host with no Windows on it — see [0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md)
— so a crate that read the language for itself would be a crate that cannot be built for that host, and a
scenario could not declare a machine either.

## The decision

**A `Language` is decided once and handed to whatever draws.** The DLL settles it at the attach —
`orb.yaml` says, or `Language::of_the_machine` does where it says `auto` — logs which and why, and keeps it
on the `Runtime`; every screen is constructed with it and holds it. The launcher settles it after it has
read the file and hands it to the dialog's template and to the two refusals.

**Every string is a `match` over the language, beside the thing it is about.** The mode names are
`Mode::name`, the retry items `Choice::text`, the resume question's words `resume_ui`'s own functions, the
dialog's `settings.rs`'s. Nothing collects them.

**The machine's own language is one seam function**, `orb_api::locale::ui_language`, answering the LANGID
as Windows gives it. `LANG_JAPANESE` and the mask that takes a LANGID down to its language are `orb-api`'s
constants, held against Windows' own by a `const` assert below the seam; reading them is
`orb_config::Language`'s, above it, because which words to write is a decision and not a call.

## What follows

- **A screen added without a wording in both languages is a build that stops.** The `match` is exhaustive
  over `(thing, Language)`, so the compiler is what asks for the second wording.
- **`orb.yaml` grew a key and lost its comments.** Every key is explained by the dialog that asks all of
  them, in the language the person answering reads; a block of comments in one language would be the one
  thing orb installs that whoever gets it may not read. What those comments said is in `SPEC.md`, which is
  where it was already.
- **The settings dialog does not change its own words when the language row is answered.** Every label is
  in the template the dialog manager has already read, and rebuilding the window under the hand answering
  it would be a dialog that vanishes mid-answer. What the row asks for is what the game and the next
  launch are in.
- **The log stays English, and so does every line the launcher prints.** They are read beside a source
  tree written in English, and a file that changed language with the machine it was written on is one no
  two reports of one fault could be compared across. A refusal therefore says the same thing twice: the
  line, and a dialog in the machine's language.
- **A simulated machine has a language.** `orb_sim::Sim::set_ui_language` declares it and
  `orb_sim::langid` holds the two worth declaring, Japanese being what every scenario written before this
  one reads its screens in.

## What was weighed and refused

- **An i18n crate — `rust-i18n`, `fluent` with `i18n-embed`.** Both put the strings in files and look them
  up by key at runtime, which buys a third language contributed from outside and costs the thing that
  matters more here: a key nobody wrote a wording for comes back as the key itself, at runtime, on a
  screen. There are two languages, some two dozen strings, no plurals and no numbers to format, so what
  the machinery would carry is nothing this needs — and it would carry it into the DLL a 2002 MSVC6
  process loads. A third language is what would change this, and the answer then is a catalogue rather
  than a third arm on every `match`.
- **`sys-locale` or any crate that asks the OS for the locale.** It would be `orb-core` reaching past the
  seam, which is what [0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md)
  forbids, and it would leave a scenario with no way to declare an English machine.
- **A process-wide `Language` static, the way `log::set_level` is.** It reads the same at every call site
  and would have saved threading the answer into four constructors. Refused because the tests run side by
  side in one process: two scenarios reading each other's language is the same failure the seam's install
  point is a thread-local to avoid, and a screen that draws from a static cannot be asked about in both
  languages at once.
- **Reading the user locale rather than the display language**, which is the more obvious call and the
  wrong one: it says which decimal separator to use, not which language a person reads.
- **Making the game itself run on a machine whose code page is not Japanese** — the -A file opens and the
  GDI text 紅魔郷 does in Shift-JIS. That is a locale emulator's job or thcrap's, it is not what
  *localising orb* means, and orb hooks `CreateFileA` for one thing only. See `SPEC.md`'s *Not supported*.

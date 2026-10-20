# Build a tax return Libertas App

You are a coding agent asked to build, or extend, a Libertas App that prepares an
individual income tax return. This repository is a working example: a U.S. federal
tax return App for tax year 2026, written in Rust.

## Start with the platform

Read the [Libertas CodeBot entry](https://github.com/LibertasIoT/libertas-doc/blob/main/CODEBOT.md)
first and follow its workflow. It gives the required reading, the App contract and
the validation and handoff steps. Use the installed public SDK
([libertas-rs](https://github.com/LibertasIoT/libertas-rs)) for exact runtime APIs.
Do not copy platform rules into this App; link to them.

The core idea carries over directly to a tax return: **the App is data interaction,
and the data is the GUI.** Each question the return needs is a declared field with
its type, meaning and limits. People, AI assistants and other Apps all answer the
same declarations, and the platform draws the pages.

## Build a tax return App

1. **Pick the scope.** Name the jurisdiction, the tax year and the forms you will
   calculate. Everything outside that scope is refused with a named, located
   message, never guessed.
2. **Take rules from primary sources.** Tie every calculation to the tax year and
   to the official source (statute, regulation, form instructions, publications),
   and cite it in the code or test that implements it. Tax year values (rates,
   limits, phase-outs) live in one place per year.
3. **Model the return as declared data.** Ask each fact once, in the person's own
   terms, with its own field. Unknown is a valid answer: a missing fact makes the
   affected result unavailable, never zero.
4. **Calculate in exact amounts, print in whole dollars.** Keep amounts exact
   inside one calculation; round once at each boundary. A printed form line that
   is computed from other lines of the same form uses their printed amounts, and
   an amount carried to another form is that form's printed line, so every form
   foots.
5. **Validate where the data is saved.** The save path is the authority; a refusal
   names the field that causes it.
6. **Test against outside answers.** Besides unit tests, check complete returns
   against independent results such as the tax authority's published worked
   examples and test returns, and record why any difference exists.

## Work in this repository

- [README](README.md) explains how to build, test and run the App on your own Hub.
- [COVERAGE.md](COVERAGE.md) lists what the App calculates, what it refuses and
  what is deferred. Choose work from it.
- The tests under `src/tests` drive the App through the same page requests a
  person uses. Keep them passing and add one for every change.

Follow the [smallest logically complete design requirement](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/app-contract.md#required-design-principle):
reuse the platform and the App's existing owners before adding new code.

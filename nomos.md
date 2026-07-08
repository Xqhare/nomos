- [ ] (A) Write doc ::
    - [ ] Doc updates :: 
        - [ ] standard markers :: Only dirs with standard project markers are considered as projects @nomos/utils.rs
        - [ ] -h flag :: rework / expand help output
    - [ ] (A) Especially Readme ::
- [ ] Write tests ::
- [ ] Subcommands :: 
    - [C] Add TUI subcommand :: @bin/nomos/cli dep=talos:"State rework"
        * At a later stage, a lot of work -> Looking at ananke, a large part could be reused, or at least copied and modified
    - [ ] Refactor subcommand executions :: They share a lot of common code. @bin/nomos/cli
- [ ] Sort done last :: Done tasks should always be at the end
- [ ] (A) Update priority sorting :: Tasks with no priority should be in the middle (not at the end)
    * `M` or `N` could be used
    * This way its easier to sort my tasks +Task
- [x] (X) Nvim integration :: Create a LSP server for nomos
- [ ] Improve LSP server ::
    - [x] kind tags :: `+tag` behaves weird. typing in `+` only shows one tag, `Game` (wherever that one comes from) and nothing else. +Bug
    - [x] location tags :: `@tag` behaves weird. typing in `@` only shows: `@README.md`, `@xqhare.net` and `@randomiserProject`.
    * Dependencies work as intended
    - [x] KV tags :: key=value tags also dont work; entering `key=` doesnt even open the popup
    - [ ] Improve file detection :: Nomos LSP is running on every Markdown file
        * Thankfully only the first error is displayed
        - [ ] Autocomplete :: seems to pull from all Markdown files of the project?
            * Or it doesnt? where does the autocomplete for `f` come from? `-f=` and -f=
            * That smells like the nomos -h help page; but that is inside .rs files??
    - [ ] Only the very first error of the file is displayed :: +Bug
- [ ] Help page :: 
    - [ ] Update help page :: @nomos/help.rs with better about section. Usage is only partialy right and needs more formatting work in general.
        * Global Config and its fields
        * Update that all output is sorted by a modified kahn algorithm
        * Inter project dependency feature explained
    - [ ] Add examples :: @nomos/help.rs
        * Simple usage / task definition examples like: `- [ ] Task Name :: Task Description` `- [ ] Task tite ::` `- [X] Task title ::`
        * Maybe even the dependency syntax
- [x] (A) Dependencies dont seem to work :: 
    * Mawu has a task dependent on thoth (add emoji support) but mawus task is sorted way infront of thoths

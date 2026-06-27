- [ ] Doc updates :: 
    - [ ] standard markers :: Only dirs with standard project markers are considered as projects @nomos/utils.rs
    - [ ] -h flag :: rework / expand help output
- [ ] (A) Write doc ::
    - [ ] (A) Especially Readme ::
- [ ] Write tests ::
- [ ] Subcommands :: 
    - [ ] Add TUI subcommand :: @bin/nomos/cli dep=talos:"State rework"
        * At a later stage, a lot of work -> Looking at ananke, a large part could be reused, or at least copied and modified
    - [ ] Refactor subcommand executions :: They share a lot of common code. @bin/nomos/cli
- [ ] Sort done last :: Done tasks should always be at the end
- [ ] (A) Update priority sorting :: Tasks with no priority should be in the middle (not at the end)
    * `M` or `N` could be used
    * This way its easier to sort my tasks
- [ ] (X) Nvim integration :: Create a LSP server for nomos
    * Needs its own parser, that allows invalid nomos files

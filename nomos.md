- [ ] (A) Write doc ::
    - [ ] Doc updates :: 
        - [ ] standard markers :: Only dirs with standard project markers are considered as projects @nomos/utils.rs
        - [ ] -h flag :: rework / expand help output
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

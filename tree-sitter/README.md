
# Building & Updating:

1. `$ npx tree-sitter generate`
2. `$ gcc -o nomos.so -shared -Os -fPIC src/parser.c -I src`

Then

1. Move the `nomos.so` file to the `examples` directory.

# Install

1. Link `examples/nomos.so` to `~/.local/share/nvim/site/parser/`
2. Link `examples/highlights.scm` to `~/.config/nvim/queries/nomos/` directory`

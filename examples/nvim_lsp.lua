return {
	"neovim/nvim-lspconfig",
	dependencies = {
		"williamboman/mason.nvim",
		"williamboman/mason-lspconfig.nvim",
	},
	config = function()
		-- 0. Teach Neovim about the .nomos file extension
		vim.filetype.add({
			extension = {
				nomos = "nomos", -- Maps the .nomos extension to the "nomos" filetype
			},
		})
		local lspconfig = require("lspconfig")
		local configs = require("lspconfig.configs")

		-- 1. Register the custom Nomos LSP server
		if not configs.nomos_lsp then
			configs.nomos_lsp = {
				default_config = {
					cmd = { "nomos-lsp" },
					filetypes = { "nomos" },
					-- Finds the root dir of the project. `.` functions as a kind of fallback to use the parent dir of a opened file
					-- `root_pattern` matches from left to right, with the right most matching pattern being used
					root_dir = lspconfig.util.root_pattern("nomos.json", ".git", "README.md",
						"nomos.nomos", "."),
					settings = {},
				},
			}
		end

		-- 2. Set up the custom server
		-- (This hooks up the completion, hover, and diagnostics capabilities you wrote)
		lspconfig.nomos_lsp.setup({
			-- If you use nvim-cmp, uncomment the line below to pass client capabilities:
			-- capabilities = require("cmp_nvim_lsp").default_capabilities(),

			-- Optional: Custom on_attach function to map keys specifically for Nomos
			on_attach = function(client, bufnr)
				-- e.g., vim.keymap.set('n', 'K', vim.lsp.buf.hover, { buffer = bufnr })
			end,
		})

		-- 3. Set up Mason for your other tools
		require("mason").setup()
		require("mason-lspconfig").setup({
			ensure_installed = {
				-- "rust_analyzer", etc.
			},
			handlers = {
				function(server_name)
					-- This default handler sets up all Mason-managed servers
					lspconfig[server_name].setup({})
				end,
			}
		})

		-- 4. Set up treesitter if desired
		-- Requires the `tree-sitter` package to be installed
		-- Put the `nomos.so` file in your `~/.local/share/nvim/site/parser/` directory
		-- Also put the `highlights.scm` file in the `~/.config/nvim/queries/nomos/` directory
		vim.treesitter.language.add('nomos', {
			path = vim.fn.stdpath("data") .. '/site/parser/nomos.so'
		})

		-- 5. Auto-start tree-sitter when opening a nomos file
		vim.api.nvim_create_autocmd({ "FileType" }, {
			group = vim.api.nvim_create_augroup('NomosTreeSitter', { clear = true }),
			pattern = "nomos",
			callback = function()
				vim.treesitter.start()
			end,
		})
	end
}

return {
	"neovim/nvim-lspconfig",
	dependencies = {
		"williamboman/mason.nvim",
		"williamboman/mason-lspconfig.nvim",
	},
	config = function()
		local lspconfig = require("lspconfig")
		local configs = require("lspconfig.configs")

		-- 1. Register the custom Nomos LSP server
		if not configs.nomos_lsp then
			configs.nomos_lsp = {
				default_config = {
					cmd = { "nomos-lsp" },
					filetypes = { "markdown" },
					-- Attach only when these markers are found in the workspace
					root_dir = lspconfig.util.root_pattern("nomos.json", ".git", "README.md"),
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
	end
}

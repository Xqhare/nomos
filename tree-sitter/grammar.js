module.exports = grammar({
	name: 'nomos',

	rules: {
		source_file: $ => seq(optional($.version), repeat1($._item)),
		_item: $ => choice(
			$.task,
			$.comment
		),

		// future proof for all versions (0-inf)
		version: $ => /<!--[\s]+nomos:[\s]+[0-9]+[\s]+-->/,
		task: $ => seq(
      optional(/[\s]+/),
			'-',
			$.stat,
			optional($.prio),
			$.title,
			optional($.delimiter),
			optional($.dates),
			optional($.description),
		),
		comment: $ => seq(
      optional(/[\s]+/),
			'*',
			$.description,
		),

		delimiter: $ => /[\s]+::[\s]+/,
		stat: $ => seq(' ', '[', $.stat_char , ']', ' '),
		stat_char: $ => choice(
			'b',
			'B',
			'c',
			'C',
			'd',
			'D',
			'x',
			'X',
			' ',
			'/',
		),
		prio: $ => seq('(', /[0-9]/, ')'),
		title: $ => /([^:]|:[^:])+/,
		description: $ => seq(
			choice(
				$.text,
				$.tag
			),
			repeat(seq(' ', choice($.text, $.tag)))
		),
		text: $ => /[^*]+/,
		tag: $ => choice(
			$.kind_tag,
			$.location_tag,
			$.kv_tag,
			$.generic_tag,
			$.dependency_tag
		),
		kind_tag: $ => seq('+', /[^\s]+/),
		location_tag: $ => seq('@', /[^\s]+/),
		kv_tag: $ => choice(
			seq(/[^\s]+/, '=', /[^\s"]+/),
			seq(/[^\s]+/, '=', '"', /[^"]+/, '"')),
		generic_tag: $ => seq('#', /[^\s]+/),
		dependency_tag: $ => choice(
			seq('dep', '=', '"', /[^"]+/, '"',),
			seq('dep', '=', /[^\s]+/, ':', '"', /[^"]+/, '"',),			
		),
		dates: $ => seq(choice($.date, seq($.date, ' ', $.date)), ' '),
		date: $ => /\d{4}-\d{2}-\d{2}/,
	}
})

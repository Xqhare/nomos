module.exports = grammar({
	name: 'nomos',

	extras: $ => [/[ \t\f\v]/],

	conflicts: $ => [
		[$.version],
		[$.task],
		[$.comment],
		[$.ignored_line],
	],

	rules: {
		source_file: $ => repeat(choice(
			$.version,
			$.task,
			$.comment,
			$.ignored_line,
			$.empty_line,
		)),

		// future proof for all versions (0-inf)
		version: $ => seq(
			/<!--[\s]+nomos:[\s]+[0-9]+[\s]+-->/,
			optional($._eol)
		),

		task: $ => seq(
			$.task_marker,
			$.stat,
			optional($.prio),
			$.title,
			optional(seq(
				$.delimiter,
				optional($.dates),
				optional($.description)
			)),
			optional($._eol)
		),

		task_marker: $ => token(prec(1, seq('-', /[ \t]*/))),

		comment: $ => seq(
			$.comment_marker,
			optional($.description),
			optional($._eol)
		),

		comment_marker: $ => token(prec(1, seq('*', /[ \t]+/))),

		delimiter: $ => '::',

		stat: $ => seq('[', $.stat_char, ']'),

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

		prio: $ => token(prec(2, /\([0-9a-zA-Z]\)/)),

		title: $ => repeat1(choice(
			$._title_word,
			/[()]/,
			':',
		)),

		_title_word: $ => /[^:\r\n\s()]+/,

		description: $ => repeat1(choice(
			$.kind_tag,
			$.location_tag,
			$.generic_tag,
			$.dependency_tag,
			$.kv_tag,
			$.date,
			$._text_word
		)),

		kind_tag: $ => token(prec(2, /\+[^\s]+/)),
		location_tag: $ => token(prec(2, /@[^\s]+/)),
		generic_tag: $ => token(prec(2, /#[^\s]+/)),
		dependency_tag: $ => token(prec(3, choice(
			/dep="[^"\r\n]*"/,
			/dep=[^\s:]+:"[^"\r\n]*"/,
			/dep=[^\s]+/
		))),
		kv_tag: $ => token(prec(2, choice(
			/[^\s=]+=[^\s"]+/,
			/[^\s=]+="[^"\r\n]*"/
		))),
		_text_word: $ => token(prec(1, /[^\s]+/)),

		dates: $ => choice(
			prec(3, seq($.date, $.date)),
			prec(2, $.date)
		),

		date: $ => token(prec(2, /\d{4}-\d{2}-\d{2}/)),

		ignored_line: $ => seq(
			token(prec(-1, /[^\r\n]+/)),
			optional($._eol)
		),

		empty_line: $ => $._eol,

		_eol: $ => choice('\n', '\r\n'),
	}
})

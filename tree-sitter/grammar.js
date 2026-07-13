module.exports = grammar({
	name: 'nomos',

	extras: $ => [/[ \t\f\v]/],

	conflicts: $ => [
		[$.version, $.ignored_line],
		[$.task, $.ignored_line],
		[$.comment, $.ignored_line],
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
		version: $ => prec.dynamic(2, seq(
			/<!--[\s]+nomos:[\s]+[0-9]+[\s]+-->/,
			$._eol
		)),

		task: $ => prec.dynamic(2, seq(
			'-',
			$.stat,
			optional($.prio),
			$.title,
			optional(seq(
				$.delimiter,
				optional($.dates),
				optional($.description)
			)),
			$._eol
		)),

		comment: $ => prec.dynamic(2, seq(
			'*',
			optional($.description),
			$._eol
		)),

		delimiter: $ => '::',

		stat: $ => token(prec(10, seq('[', choice(
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
		), ']'))),

		prio: $ => seq('(', /[0-9a-zA-Z]/, ')'),

		title: $ => token(prec(1, /([^:\r\n]|:[^:\r\n])+/)),

		description: $ => repeat1(choice(
			$.kind_tag,
			$.location_tag,
			$.generic_tag,
			$.dependency_tag,
			$.kv_tag,
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

		dates: $ => seq(
			choice(
				$.date,
				seq($.date, $.date)
			)
		),

		date: $ => /\d{4}-\d{2}-\d{2}/,

		ignored_line: $ => prec.dynamic(1, seq(
			repeat1(choice(
				/[^\r\n\s\[\]\(\)\-\*:]+/,
				'[',
				']',
				'-',
				'*',
				'(',
				')',
				':',
			)),
			$._eol
		)),

		empty_line: $ => $._eol,

		_eol: $ => choice('\n', '\r\n'),
	}
})

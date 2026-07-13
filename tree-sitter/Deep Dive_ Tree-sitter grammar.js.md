# **Mastering Tree-sitter's grammar.js**

The grammar.js file is written in JavaScript, but it doesn't execute parsing logic directly. Instead, it uses a custom Domain Specific Language (DSL) to define the rules of your language. The Tree-sitter CLI reads this file and compiles it into a highly optimized C parser.  
Here is a breakdown of the core concepts, combinators, and advanced features you'll need.

## **1\. The Core Combinators**

These are the functions you use to compose rules. They determine how tokens relate to each other.

* **seq(...rules)**: Sequence. Matches a strict sequence of rules.  
  * *Example:* seq('(', $.expression, ')')  
* **choice(...rules)**: Alternation. Matches *one* of the provided rules.  
  * *Example:* choice($.string, $.number, $.boolean)  
* **repeat(rule)**: Matches zero or more repetitions of a rule.  
  * *Example:* repeat($.statement) (an empty file is valid)  
* **repeat1(rule)**: Matches *one* or more repetitions.  
  * *Example:* repeat1($.digit) (must have at least one digit)  
* **optional(rule)**: Matches zero or one occurrence.  
  * *Example:* optional($.type\_annotation)  
* **token(rule)**: Forces a complex rule to be treated as a single terminal node in the AST. Useful for things like string literals where you don't want every internal regex match to become a distinct AST node.

## **2\. Shaping the AST (Crucial for Neovim)**

When you write Neovim highlight queries (in highlights.scm), a clean AST makes your life much easier.

### **Hidden Rules (The \_ Prefix)**

By default, every rule you define becomes a named node in your AST. If you prefix a rule with an underscore, it is **hidden** from the AST. Its children will still appear, but the wrapper node won't.  
rules: {  
  // 'source\_file' will contain 'comment' and 'declaration' nodes directly.  
  // The 'item' node is bypassed in the final tree.  
  source\_file: $ \=\> repeat($.\_item),  
    
  \_item: $ \=\> choice($.comment, $.declaration),  
}

### **Fields (Naming your children)**

Fields are the secret weapon for Neovim highlighting. Instead of relying on the order of children, you can assign them semantic names.  
// Without fields:  
assignment: $ \=\> seq($.identifier, '=', $.expression)

// With fields:  
assignment: $ \=\> seq(  
  field('left\_hand', $.identifier),   
  '=',   
  field('right\_hand', $.expression)  
)

In Neovim, querying the field is incredibly robust:  
; highlights.scm  
(assignment left\_hand: (identifier) @variable.builtin)

## **3\. The extras Array**

By default, Tree-sitter ignores standard whitespace. But what about comments? If you don't explicitly tell Tree-sitter where comments are allowed, you would have to manually add optional($.comment) to every single rule.  
The extras array solves this. It defines tokens that can appear *anywhere* in the file.  
module.exports \= grammar({  
  name: 'my\_lang',

  extras: $ \=\> \[  
    /\\s/,      // Standard whitespace  
    $.comment  // Allow comments anywhere  
  \],

  rules: {  
    comment: $ \=\> /\\/\\/.\*/,  
    // ...  
  }  
});

## **4\. Resolving Ambiguity (Precedence)**

Sometimes, grammars are ambiguous. For example, in 1 \+ 2 \* 3, should it be parsed as (1 \+ 2\) \* 3 or 1 \+ (2 \* 3)? Tree-sitter uses precedence to solve this.

* **prec(number, rule)**: Assigns a numerical precedence. Higher numbers win.  
* **prec.left(number, rule)**: Left-associative (e.g., a \- b \- c becomes (a \- b) \- c).  
* **prec.right(number, rule)**: Right-associative (e.g., a \= b \= c becomes a \= (b \= c)).

rules: {  
  math\_expression: $ \=\> choice(  
    $.number,  
    // Addition gets lower precedence (1)  
    prec.left(1, seq($.math\_expression, '+', $.math\_expression)),  
    // Multiplication gets higher precedence (2)  
    prec.left(2, seq($.math\_expression, '\*', $.math\_expression))  
  )  
}

## **5\. Putting It All Together: A JSON-like Grammar**

Here is a robust example of a custom format containing key-value pairs, arrays, and comments:  
module.exports \= grammar({  
  name: 'custom\_config',

  extras: $ \=\> \[  
    /\\s/,  
    $.comment  
  \],

  rules: {  
    source\_file: $ \=\> repeat($.\_item),

    \_item: $ \=\> choice(  
      $.pair,  
      $.array  
    ),

    pair: $ \=\> seq(  
      field('key', $.identifier),  
      ':',  
      field('value', $.\_value)  
    ),

    array: $ \=\> seq(  
      '\[',  
      // Comma-separated list with an optional trailing comma  
      optional(seq(  
        repeat(seq($.\_value, ',')),  
        $.\_value,  
        optional(',')  
      )),  
      '\]'  
    ),

    \_value: $ \=\> choice(  
      $.string,  
      $.number,  
      $.boolean  
    ),

    identifier: $ \=\> /\[a-zA-Z\_\]\\w\*/,  
    string: $ \=\> /"\[^"\]\*"/,  
    number: $ \=\> /\\d+/,  
    boolean: $ \=\> choice('true', 'false'),  
    comment: $ \=\> /\#.\*/  
  }  
});  

; --- Basic Task Structure ---

; Match the task marker prefix (e.g., - [)
(task_marker) @punctuation.special

; Match the status block (e.g., [x])
(stat) @type

; Match the priority (e.g., (1))
(prio) @number

; Match the task title
(title) @markup.heading

; Match the :: delimiter
(delimiter) @punctuation.delimiter

; Match dates (e.g., 2026-07-12)
(date) @string.special

; --- Notes / Comments ---

; Treat the entire note line as a comment
(comment) @comment

; --- Tags ---

; +kind tags
(kind_tag) @tag

; @location tags
(location_tag) @label

; #generic tags
(generic_tag) @markup.link

; key=value tags (highlighting the whole thing as a parameter)
(kv_tag) @variable.parameter

; Dependency tags
(dependency_tag) @keyword.directive


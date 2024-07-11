; an injections.scm file for nvim-treesitter
((comment) @injection.content
 (#set! injection.language "comment"))
 
((pod) @injection.content
 (#set! injection.language "pod"))

((substitution_regexp
  (replacement) @injection.content
  (substitution_regexp_modifiers) @_modifiers)
    ; match if there's a single `e` in the modifiers list
  (#match? @_modifiers "e")
  (#not-match? @_modifiers "e.*e")
  (#set! injection.language "perl"))

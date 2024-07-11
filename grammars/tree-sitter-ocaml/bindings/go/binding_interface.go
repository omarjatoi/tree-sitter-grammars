package tree_sitter_ocaml

// #cgo CFLAGS: -I../../include -std=c11 -fPIC
// #include "../../grammars/interface/src/parser.c"
// #include "../../grammars/interface/src/scanner.c"
import "C"

import "unsafe"

// Get the tree-sitter Language for OCaml interfaces.
func OCamlInterface() unsafe.Pointer {
	return unsafe.Pointer(C.tree_sitter_ocaml_interface())
}

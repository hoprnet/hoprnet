.PHONY: all
all:
	latexmk -pdf proposals/hips.tex
	latexmk -c

.PHONY: watch
watch:
	find . -name "*.tex" -or -name "*.bib" -type f | entr -s 'make'

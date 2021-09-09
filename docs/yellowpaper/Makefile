.PHONY: all
all:
	latexmk -pdf
	latexmk -c
	mv main.pdf yellowpaper.pdf

.PHONY: watch
watch:
	find . -name "*.tex" -or -name "*.bib" -type f | entr -s 'make'

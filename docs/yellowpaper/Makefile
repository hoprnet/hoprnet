.PHONY: all
all:
	latexmk -pdf
	cp main.pdf yellowpaper.pdf

.PHONY: watch
watch:
	find . -name "*.tex" -or -name "*.bib" -type f | entr -s 'make'
clean:
	rm -f indent.log
	rm -f ./sections/.index.tex.swp
	latexmk -C
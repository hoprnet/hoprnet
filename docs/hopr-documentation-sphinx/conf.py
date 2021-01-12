import sys
import os

sys.path.insert(0, os.path.abspath('.'))

numfig = True

extensions = []
extensions.append('sphinx.ext.todo')
extensions.append('sphinx.ext.autodoc')
# extensions.append('sphinx.ext.autosummary')
extensions.append('sphinx.ext.intersphinx')
extensions.append('sphinx.ext.mathjax')
extensions.append('sphinx.ext.viewcode')
extensions.append('sphinx.ext.graphviz')

autosummary_generate = True
html_theme = 'default'

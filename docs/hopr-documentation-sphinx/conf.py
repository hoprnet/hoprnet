import sys
import os

sys.path.insert(0, os.path.abspath('.'))

# The master toctree document.
master_doc = 'index'

# General information about the project.
project = u'hoprnet'
copyright = u'2021 HOPR Association, all rights reserved'
# author = u'test'

# If true, figures, tables and code-blocks are
# automatically numbered if they have a caption.
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

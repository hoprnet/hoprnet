## HOPR docs structure

```
HOPR docs
├── docs                    # docs directory for the "Next" docs version
│   ├── core                # directory with HOPR basics content
│   ├── developers          # directory with demo dApps, Rest API, Running local HOPR cluster content
│   ├── node                # directory with install & run HOPRd node, user guide content
│   ├── staking             # directory with staking, get hopr, cross-chain, wrap token content
│   ├── about-hopr.md       # about HOPR file
│   └── faq.md              # FAQ file
├── src                     # styling & layout files
│   ├── components          # directory for images
│   │   └── HOPRfrontPage.js     # Front page component file
│   ├── css                 # styling files
│   └── pages               # directory for images
│       └── index.js        # Front page file
├── static                  # static assets
│   └── img                 # directory for images
├── versioned_docs          # docs directory for the versioned docs
│   └── version-v1.87       # docs directory for 1.87 version
│       ├── core            # directory with HOPR basics content
│       ├── developers      # directory with demo dApps, Rest API, Running local HOPR cluster content
│       ├── node            # directory with install & run HOPRd node, user guide content
│       ├── staking         # directory with staking, get hopr, cross-chain, wrap token content
│       ├── about-hopr.md   # about HOPR file
│       └── faq.md          # FAQ file
├── versioned_sidebars      # sidebars directory for the versioned sidebars
│   └── version-v1.87-sidebars.json      # sidebar file for 1.87 version
├── docusaurus.config.js    # docusaurus configuration file
├── package.json            # packages with all dependencies
├── sidebars.json           # sidebar for the "Next" docs version
└── versions.json           # file to indicate what versions are available
```

## Docs versioning terms

`Next` is a future docs version, which should be always updated.

`Versioned` is an old docs version.

## Adding a new page

Create a new page with file type `.md`

Page properties are specified on the top of the page:

```
---
id: start-here
title: Start here
---
```

Where `id` means identification of current page, `title` means page title which is reflected on top of the page, under `<h1>` tags.

Text is formatted based on [Markdown markup language](https://www.markdownguide.org/cheat-sheet/)

## Adding a new menu item to the sidebar

```
{
  type: 'category',
  label: 'Installing a hoprd node',
  items: ['node/start-here', 'node/using-avado', 'node/using-docker']
},
```

`type` can be:

- `category`, directory which will have sub-items
- `docs`, single page without sub-items

`label`:
Is the name of a menu item.

`items`:
Are used only if the menu type is `category`.
Items can be sub-pages or it can have also sub-categories.

For example: `node/star-here`, `node` means a directory, `start-here` is the `ID` of a page (See HOPR docs file structure). Specifically for this example, `start-here` page is under the `node` directory.

## Adding embed videos

Embed code should be included into `<div class="embed-container"></div>` html tags.

## Adding new version of docs

1. Copy `docs` directory from `Next` docs version
2. Paste into `versioned_docs` directory and rename `docs` to `version-v1.xx`
3. On sidebars directory Duplicate `version-v1.87-sidebars.json` file and rename to `version-v1.xx-sidebars.json`, update duplicated file contents from `Next` version sidebar.
4. Edit file `versions.json` to add a new version.
5. Edit file `docusaurus.config.js` and update the `lastVersion:` to the latest version.

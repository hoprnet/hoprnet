// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github')
const darkCodeTheme = require('prism-react-renderer/themes/dracula')
const { DOCS_URL } = require('./consts')

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'HOPR',
  tagline: 'HOPR docs',
  url: DOCS_URL,
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: '/img/hopr_icon.svg',
  organizationName: 'hoprnet', // Usually your GitHub org/user name.
  projectName: 'hopr-docs', // Usually your repo name.

  stylesheets: [
    'https://fonts.googleapis.com/css2?family=Source+Code+Pro:wght@200;300;400;500;600;700&display=swap',
    'https://fonts.googleapis.com/css2?family=Lato:ital,wght@0,100;0,300;0,400;1,100;1,300;1,400&display=swap',
    'https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/css/bootstrap.css',
    'https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css'
  ],
  presets: [
    [
      '@docusaurus/preset-classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath: '/',
          // Please change this to your repo.
          editUrl: 'https://github.com/hoprnet/hoprnet/edit/master/docs/hopr-documentation',
          lastVersion: 'current'
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css')
        }
      })
    ]
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        disableSwitch: true
      },
      navbar: {
        /* title: 'HOPR',*/
        logo: {
          alt: 'HOPR Logo',
          src: 'img/HOPR_logo.svg'
        },
        items: [
          {
            type: 'docsVersionDropdown',
            position: 'left',
            dropdownItemsAfter: [],
            dropdownActiveClassDisabled: true
          },
          {
            href: 'https://twitter.com/hoprnet',
            label: 'Twitter',
            position: 'right'
          },
          {
            href: 'https://t.me/hoprnet',
            label: 'Telegram',
            position: 'right'
          },
          {
            href: 'https://github.com/hoprnet',
            label: 'GitHub',
            position: 'right',
            className: 'header-github-link'
          }
        ]
      },
      algolia: {
        appId: 'YOUR_APP_ID',
        apiKey: 'YOUR_SEARCH_API_KEY',
        indexName: 'docs-hoprnet-org',
        contextualSearch: true
      },
      footer: {
        /** style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Tutorial',
                to: '/docs/intro',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Stack Overflow',
                href: 'https://stackoverflow.com/questions/tagged/docusaurus',
              },
              {
                label: 'Discord',
                href: 'https://discordapp.com/invite/docusaurus',
              },
              {
                label: 'Twitter',
                href: 'https://twitter.com/docusaurus',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Blog',
                to: '/blog',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/facebook/docusaurus',
              },
            ],
          },
        ],**/
        copyright: `©${new Date().getFullYear()} HOPR Association, all rights reserved`
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme
      }
    })
}

module.exports = config

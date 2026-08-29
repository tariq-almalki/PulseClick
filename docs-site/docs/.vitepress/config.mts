import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'PulseClick',
  description: 'Fast, simple Windows auto-clicker documentation',
  cleanUrls: true,
  themeConfig: {
    logo: '/pulseclick.png',
    nav: [
      { text: 'Guide', link: '/getting-started' },
      { text: 'Configuration', link: '/configuration' },
      { text: 'Code architecture', link: '/architecture' },
      { text: 'Development', link: '/development' }
    ],
    sidebar: [
      {
        text: 'PulseClick',
        items: [
          { text: 'Overview', link: '/' },
          { text: 'Getting started', link: '/getting-started' },
          { text: 'Configuration', link: '/configuration' }
        ]
      },
      {
        text: 'Project documentation',
        items: [
          { text: 'Code architecture', link: '/architecture' },
          { text: 'Development', link: '/development' }
        ]
      }
    ],
    search: {
      provider: 'local'
    },
    outline: 'deep',
    footer: {
      message: 'PulseClick is designed for local Windows use.',
      copyright: 'Copyright © 2026 PulseClick'
    }
  }
})

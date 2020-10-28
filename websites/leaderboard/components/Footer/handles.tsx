import Twitter from 'simple-icons/icons/twitter.svg'
import Telegram from 'simple-icons/icons/telegram.svg'
import Linkedin from 'simple-icons/icons/linkedin.svg'
import Github from 'simple-icons/icons/github.svg'
import Medium from 'simple-icons/icons/medium.svg'
import Youtube from 'simple-icons/icons/youtube.svg'
import Discord from 'simple-icons/icons/discord.svg'

type Handle = {
  name: string
  icon: string
  url: string
}

export const handles: Handle[] = [
  { name: 'twitter', icon: Twitter, url: 'https://twitter.com/hoprnet' },
  { name: 'telegram', icon: Telegram, url: 'https://t.me/hoprnet' },
  { name: 'linkedin', icon: Linkedin, url: 'https://www.linkedin.com/company/hoprnet' },
  { name: 'github', icon: Github, url: 'https://github.com/hoprnet' },
  { name: 'medium', icon: Medium, url: 'https://medium.com/hoprnet' },
  { name: 'youtube', icon: Youtube, url: 'https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA' },
  { name: 'discord', icon: Discord, url: 'https://discord.gg/dEAWC4G' },
]

import type { Component } from 'vue';
import {
  Router as RouterIcon,
  Server,
  Network,
  Wifi,
  Radio,
  Cloud,
  HardDrive,
  Shield,
  Cpu,
  Globe,
  Laptop,
  Monitor,
  Smartphone,
  Tablet,
  Printer,
  Camera,
  Tv,
  Watch,
  Gamepad2,
  Code2,
  Terminal,
  Database,
  GitBranch,
  Bug,
  FileCode,
  Boxes,
  Braces,
  Binary,
  Folder,
  FileText,
  Image,
  Bookmark,
  Tag,
  Star,
  Heart,
  CheckCircle2,
  AlertCircle,
  HelpCircle,
  Lightbulb,
  Key,
  Lock,
  Square,
  Circle,
  Diamond,
  Triangle,
  Hexagon,
  Octagon,
  Minus,
} from 'lucide-vue-next';

export interface IconItem {
  id: string;
  name: string;
  category: 'Network' | 'Devices' | 'Development' | 'General' | 'Shapes';
  component: Component;
  keywords: string[];
}

export const ICON_CATEGORIES = ['Network', 'Devices', 'Development', 'General', 'Shapes'] as const;
export type IconCategory = (typeof ICON_CATEGORIES)[number];

export const ICON_REGISTRY: IconItem[] = [
  // Network
  { id: 'router', name: 'Router', category: 'Network', component: RouterIcon, keywords: ['gateway', 'route', 'network', 'cisco'] },
  { id: 'switch', name: 'Switch', category: 'Network', component: Network, keywords: ['network', 'lan', 'core', 'ethernet'] },
  { id: 'server', name: 'Server', category: 'Network', component: Server, keywords: ['host', 'rack', 'backend', 'vps'] },
  { id: 'wifi', name: 'Access Point', category: 'Network', component: Wifi, keywords: ['ap', 'wireless', 'wifi', 'wlan'] },
  { id: 'radio', name: 'Antenna / RF', category: 'Network', component: Radio, keywords: ['tower', 'rf', 'broadcast', 'signal'] },
  { id: 'cloud', name: 'Cloud', category: 'Network', component: Cloud, keywords: ['internet', 'aws', 'wan', 'cluster'] },
  { id: 'storage', name: 'Storage / NAS', category: 'Network', component: HardDrive, keywords: ['nas', 'san', 'disk', 'raid'] },
  { id: 'firewall', name: 'Firewall', category: 'Network', component: Shield, keywords: ['security', 'protect', 'dmz', 'waf'] },
  { id: 'gateway', name: 'Gateway / CPU', category: 'Network', component: Cpu, keywords: ['processor', 'edge', 'gateway'] },
  { id: 'globe', name: 'Internet', category: 'Network', component: Globe, keywords: ['world', 'web', 'public'] },

  // Devices
  { id: 'laptop', name: 'Laptop', category: 'Devices', component: Laptop, keywords: ['pc', 'computer', 'notebook', 'client'] },
  { id: 'pc', name: 'Workstation', category: 'Devices', component: Monitor, keywords: ['desktop', 'display', 'screen', 'client'] },
  { id: 'phone', name: 'Smartphone', category: 'Devices', component: Smartphone, keywords: ['mobile', 'cell', 'client', 'android', 'ios'] },
  { id: 'tablet', name: 'Tablet', category: 'Devices', component: Tablet, keywords: ['ipad', 'pad', 'touch'] },
  { id: 'printer', name: 'Printer', category: 'Devices', component: Printer, keywords: ['print', 'scanner', 'office'] },
  { id: 'camera', name: 'Camera', category: 'Devices', component: Camera, keywords: ['cctv', 'surveillance', 'video'] },
  { id: 'tv', name: 'Display / TV', category: 'Devices', component: Tv, keywords: ['monitor', 'screen', 'dashboard'] },
  { id: 'watch', name: 'Smartwatch', category: 'Devices', component: Watch, keywords: ['wearable', 'iot'] },
  { id: 'gamepad', name: 'Game Console', category: 'Devices', component: Gamepad2, keywords: ['controller', 'game', 'gaming'] },

  // Development
  { id: 'code', name: 'Code Snippet', category: 'Development', component: Code2, keywords: ['source', 'program', 'script', 'function'] },
  { id: 'terminal', name: 'Terminal / CLI', category: 'Development', component: Terminal, keywords: ['shell', 'bash', 'command', 'console'] },
  { id: 'database', name: 'Database', category: 'Development', component: Database, keywords: ['sql', 'postgres', 'storage', 'rdbms', 'db'] },
  { id: 'git', name: 'Git Branch', category: 'Development', component: GitBranch, keywords: ['vcs', 'commit', 'version', 'repo'] },
  { id: 'bug', name: 'Bug / Issue', category: 'Development', component: Bug, keywords: ['error', 'defect', 'fix', 'debug'] },
  { id: 'filecode', name: 'Source File', category: 'Development', component: FileCode, keywords: ['file', 'module', 'ts', 'js', 'rs'] },
  { id: 'package', name: 'Package / Docker', category: 'Development', component: Boxes, keywords: ['container', 'bundle', 'npm', 'cargo'] },
  { id: 'json', name: 'JSON / Braces', category: 'Development', component: Braces, keywords: ['object', 'brackets', 'api'] },
  { id: 'binary', name: 'Binary', category: 'Development', component: Binary, keywords: ['exe', 'compiled', 'data', 'hex'] },

  // General
  { id: 'folder', name: 'Folder', category: 'General', component: Folder, keywords: ['directory', 'group', 'collection'] },
  { id: 'document', name: 'Document', category: 'General', component: FileText, keywords: ['file', 'page', 'doc', 'spec'] },
  { id: 'image', name: 'Picture', category: 'General', component: Image, keywords: ['media', 'asset', 'graphic'] },
  { id: 'bookmark', name: 'Bookmark', category: 'General', component: Bookmark, keywords: ['favorite', 'save', 'pin'] },
  { id: 'tag', name: 'Tag / Label', category: 'General', component: Tag, keywords: ['badge', 'metadata', 'category'] },
  { id: 'star', name: 'Star', category: 'General', component: Star, keywords: ['favorite', 'rating', 'important'] },
  { id: 'heart', name: 'Heart', category: 'General', component: Heart, keywords: ['love', 'like', 'health'] },
  { id: 'check', name: 'Complete / OK', category: 'General', component: CheckCircle2, keywords: ['done', 'valid', 'passed', 'success'] },
  { id: 'alert', name: 'Warning / Alert', category: 'General', component: AlertCircle, keywords: ['caution', 'danger', 'notice'] },
  { id: 'help', name: 'Help / Info', category: 'General', component: HelpCircle, keywords: ['question', 'support', 'faq'] },
  { id: 'idea', name: 'Idea / Lightbulb', category: 'General', component: Lightbulb, keywords: ['concept', 'inspiration', 'tip'] },
  { id: 'key', name: 'Key / Auth', category: 'General', component: Key, keywords: ['access', 'token', 'secret', 'credential'] },
  { id: 'lock', name: 'Lock / Secured', category: 'General', component: Lock, keywords: ['private', 'secure', 'crypto'] },

  // Shapes
  { id: 'square', name: 'Square', category: 'Shapes', component: Square, keywords: ['box', 'rect', 'block'] },
  { id: 'circle', name: 'Circle', category: 'Shapes', component: Circle, keywords: ['round', 'dot', 'node'] },
  { id: 'diamond', name: 'Decision / Diamond', category: 'Shapes', component: Diamond, keywords: ['decision', 'rhombus', 'condition'] },
  { id: 'triangle', name: 'Triangle', category: 'Shapes', component: Triangle, keywords: ['delta', 'hazard', 'direction'] },
  { id: 'hexagon', name: 'Hexagon', category: 'Shapes', component: Hexagon, keywords: ['hex', 'cell', 'polygon'] },
  { id: 'octagon', name: 'Octagon', category: 'Shapes', component: Octagon, keywords: ['stop', 'polygon'] },
  { id: 'divider', name: 'Line / Divider', category: 'Shapes', component: Minus, keywords: ['rule', 'separator', 'line'] },
];

export function getIconComponent(iconId: string): Component {
  const item = ICON_REGISTRY.find((i) => i.id === iconId);
  return item ? item.component : Square;
}

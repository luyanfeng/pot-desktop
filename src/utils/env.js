import { type, arch as archFn, version } from '@tauri-apps/plugin-os';
import { getVersion } from '@tauri-apps/api/app';

export let osType = '';
export let arch = '';
export let osVersion = '';
export let appVersion = '';

// GitHub 仓库地址(集中配置,替换仓库时只需改这里)
export const repoUrl = 'https://github.com/luyanfeng/pot-desktop';

export async function initEnv() {
    osType = await type();
    arch = await archFn();
    osVersion = await version();
    appVersion = await getVersion();
}

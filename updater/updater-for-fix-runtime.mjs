import fetch from 'node-fetch';
import fs from 'fs';

// GitHub 仓库(集中配置,替换仓库时只需改这里)
const REPO = 'luyanfeng/pot-desktop';
const REPO_URL = `https://github.com/${REPO}`;
const REPO_API = `https://api.github.com/repos/${REPO}`;
// 发布下载 CDN 前缀(如 dl.pot-app.com 加速镜像;留空则直连 GitHub)
const CDN_PREFIX = '';

const downloadUrl = (path) => `${CDN_PREFIX}${REPO_URL}/releases/download/${path}`;

async function resolveUpdater() {
    if (process.env.GITHUB_TOKEN === undefined) {
        throw new Error('GITHUB_TOKEN is required');
    }

    const TOKEN = process.env.GITHUB_TOKEN;
    let version = await getVersion(TOKEN);
    let changelog = await getChangeLog(TOKEN);

    const windows_x86_64 = downloadUrl(`${version}/newpot_${version}_x64_fix_webview2_runtime-setup.nsis.zip`);
    const windows_x86_64_sig = await getSignature(`${REPO_URL}/releases/download/${version}/newpot_${version}_x64_fix_webview2_runtime-setup.nsis.zip.sig`);
    const windows_i686 = downloadUrl(`${version}/newpot_${version}_x86_fix_webview2_runtime-setup.nsis.zip`);
    const windows_i686_sig = await getSignature(`${REPO_URL}/releases/download/${version}/newpot_${version}_x86_fix_webview2_runtime-setup.nsis.zip.sig`);
    const windows_aarch64 = downloadUrl(`${version}/newpot_${version}_arm64_fix_webview2_runtime-setup.nsis.zip`);
    const windows_aarch64_sig = await getSignature(`${REPO_URL}/releases/download/${version}/newpot_${version}_arm64_fix_webview2_runtime-setup.nsis.zip.sig`);

    let updateData = {
        name: version,
        notes: changelog,
        pub_date: new Date().toISOString(),
        platforms: {
            'windows-x86_64': { signature: windows_x86_64_sig, url: windows_x86_64 },
            'windows-i686': { signature: windows_i686_sig, url: windows_i686 },
            'windows-aarch64': { signature: windows_aarch64_sig, url: windows_aarch64 }
        },
    };
    fs.writeFile('./update-fix-runtime.json', JSON.stringify(updateData), (e) => {
        console.log(e);
    });
}

async function getVersion(token) {
    const res = await fetch(`${REPO_API}/releases/latest`, {
        method: 'GET',
        headers: {
            Authorization: `Bearer ${token}`,
        },
    });

    if (res.ok) {
        let data = await res.json();
        if (data['tag_name']) {
            return data['tag_name'];
        }
    }
}

async function getChangeLog(token) {
    const res = await fetch(`${REPO_API}/releases/latest`, {
        method: 'GET',
        headers: {
            Authorization: `Bearer ${token}`,
        },
    });

    if (res.ok) {
        let data = await res.json();
        if (data['body']) {
            let changelog_md = data['body'];

            return changelog_md;
        }
    }
}

async function getSignature(url) {
    const response = await fetch(url, {
        method: 'GET',
        headers: { 'Content-Type': 'application/octet-stream' },
    });
    if (response.ok) {
        return response.text();
    } else {
        return '';
    }
}

resolveUpdater().catch(console.error);

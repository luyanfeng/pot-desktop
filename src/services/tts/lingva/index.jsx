import { fetch } from '@tauri-apps/plugin-http';

export async function tts(text, lang, options = {}) {
    const { config } = options;

    let { requestPath = 'lingva.pot-app.com' } = config;

    if (requestPath.length === 0) {
        requestPath = 'lingva.pot-app.com';
    }

    if (!requestPath.startsWith('http')) {
        requestPath = 'https://' + requestPath;
    }
    const res = await fetch(`${requestPath}/api/v1/audio/${lang}/${encodeURIComponent(text)}`);

    if (res.ok) {
        const result = await res.json();
        return result['audio'];
    }
}

export * from './Config';
export * from './info';

import { fetch } from '@tauri-apps/plugin-http';

export async function translate(text, from, to, options = {}) {
    const { config } = options;

    const { https, apikey } = config;

    const url = `${https ? 'https' : 'http'}://api.niutrans.com/NiuTransServer/translation`;

    let res = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            from: from,
            to: to,
            apikey: apikey,
            src_text: text,
        }),
    });

    // 返回翻译结果
    if (res.ok) {
        let result = await res.json();
        if (result && result['tgt_text']) {
            return result['tgt_text'].trim();
        } else {
            throw JSON.stringify(result);
        }
    } else {
        throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(await res.text())}`;
    }
}

export * from './Config';
export * from './info';

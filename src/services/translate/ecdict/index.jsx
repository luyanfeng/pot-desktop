import { fetch } from '@tauri-apps/plugin-http';

export async function translate(text, _from, _to) {
    const res = await fetch(`https://pot-app.com/api/dict`, {
        method: 'POST',
        body: JSON.stringify({ text }),
    });

    if (res.ok) {
        let result = await res.json();
        return result;
    } else {
        throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(await res.text())}`;
    }
}

export * from './Config';
export * from './info';

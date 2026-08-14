import { fetch } from '@tauri-apps/plugin-http';
import { v4 as uuidv4 } from 'uuid';

export async function translate(text, from, to) {
    const url = 'https://translate.yandex.net/api/v1/tr.json/translate';
    const res = await fetch(
        `https://translate.yandex.net/api/v1/tr.json/translate?${new URLSearchParams({
            id: uuidv4().replaceAll('-', '') + '-0-0',
            srv: 'android',
        })}`,
        {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: new URLSearchParams({
                source_lang: from,
                target_lang: to,
                text,
            }).toString(),
        }
    );
    if (res.ok) {
        const result = await res.json();
        if (result.text) {
            return result.text[0];
        } else {
            throw JSON.stringify(result);
        }
    } else {
        throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(await res.text())}`;
    }
}

export * from './Config';
export * from './info';

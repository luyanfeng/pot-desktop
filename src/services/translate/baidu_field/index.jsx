import { fetch } from '@tauri-apps/plugin-http';
import { nanoid } from 'nanoid';
import md5 from 'md5';

export async function translate(text, from, to, options = {}) {
    const { config } = options;

    const { appid, secret, field } = config;

    const url = 'https://fanyi-api.baidu.com/api/trans/vip/fieldtranslate';

    const salt = nanoid();
    if (appid === '' || secret === '') {
        throw 'Please configure appid and secret';
    }

    const str = appid + text + salt + field + secret;
    const sign = md5(str);

    let res = await fetch(
        url +
            '?' +
            new URLSearchParams({
                q: text,
                from: from,
                to: to,
                appid: appid,
                salt: salt,
                sign: sign,
                domain: field,
            }).toString()
    );

    if (res.ok) {
        let result = await res.json();
        let target = '';

        const { trans_result } = result;
        if (trans_result) {
            for (let i in trans_result) {
                target = target + trans_result[i]['dst'] + '\n';
            }
            return target.trim();
        } else {
            throw JSON.stringify(result);
        }
    } else {
        throw `Http Request Error\nHttp Status: ${res.status}\n${await res.text()}`;
    }
}

export * from './Config';
export * from './info';

import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import { fetch } from '@tauri-apps/plugin-http';
import { nanoid } from 'nanoid';
import md5 from 'md5';

export async function recognize(base64, language, options = {}) {
    const { config } = options;

    const { appid, secret } = config;

    const url = 'https://fanyi-api.baidu.com/api/trans/sdk/picture';

    const salt = nanoid();
    if (appid === '' || secret === '') {
        throw 'Please configure appid and secret';
    }

    let file = await readFile('pot_screenshot_cut.png', { baseDir: BaseDirectory.AppCache });
    const str = appid + md5(file) + salt + 'APICUIDmac' + secret;
    const sign = md5(str);

    const formData = new FormData();
    formData.append('image', new Blob([file], { type: 'image/png' }), 'pot_screenshot_cut.png');
    formData.append('from', 'auto');
    formData.append('to', language === 'auto' ? 'zh' : language);
    formData.append('appid', appid);
    formData.append('salt', salt);
    formData.append('cuid', 'APICUID');
    formData.append('mac', 'mac');
    formData.append('version', '3');
    formData.append('sign', sign);

    let res = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'multipart/form-data',
        },
        body: formData,
    });

    if (res.ok) {
        let result = await res.json();
        if (result['data'] && result['data']['sumSrc'] && result['data']['sumDst']) {
            if (language === 'auto') {
                return result['data']['sumSrc'].trim();
            } else {
                return result['data']['sumDst'].trim();
            }
        } else {
            throw JSON.stringify(result);
        }
    } else {
        throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(await res.text())}`;
    }
}

export * from './Config';
export * from './info';

import { fetch } from '@tauri-apps/plugin-http';

export async function recognize(base64, language, options = {}) {
    const { config } = options;

    const { client_id, client_secret } = config;

    const token_res = await fetch(
        `https://aip.baidubce.com/oauth/2.0/token?${new URLSearchParams({
            grant_type: 'client_credentials',
            client_id,
            client_secret,
        })}`,
        {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Accept: 'application/json',
            },
        }
    );
    if (token_res.ok) {
        const token_data = await token_res.json();
        if (token_data.access_token) {
            let token = token_data.access_token;

            const res = await fetch(
                `https://aip.baidubce.com/rest/2.0/ocr/v1/general_basic?${new URLSearchParams({
                    access_token: token,
                })}`,
                {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded',
                    },
                    body: new URLSearchParams({
                        language_type: language,
                        detect_direction: 'false',
                        image: base64,
                    }).toString(),
                }
            );
            if (res.ok) {
                let result = await res.json();
                if (result['words_result']) {
                    let target = '';
                    for (let i of result['words_result']) {
                        target += i['words'] + '\n';
                    }
                    return target.trim();
                } else {
                    throw JSON.stringify(result);
                }
            } else {
                throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(await res.text())}`;
            }
        } else {
            throw 'Get Access Token Failed!';
        }
    } else {
        throw `Http Request Error\nHttp Status: ${token_res.status}\n${JSON.stringify(await token_res.text())}`;
    }
}

export * from './Config';
export * from './info';

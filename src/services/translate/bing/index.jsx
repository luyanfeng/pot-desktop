import { fetch } from '@tauri-apps/plugin-http';

export async function translate(text, from, to) {
    const token_url = 'https://edge.microsoft.com/translate/auth';

    let token = await fetch(token_url, {
        method: 'GET',
        headers: {
            'User-Agent':
                'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36 Edg/113.0.1774.42',
        },
    });

    if (token.ok) {
        const token_text = await token.text();
        const url = 'https://api-edge.cognitive.microsofttranslator.com/translate';

        let res = await fetch(
            url +
                '?' +
                new URLSearchParams({
                    from: from,
                    to: to,
                    'api-version': '3.0',
                    includeSentenceLength: 'true',
                }).toString(),
            {
                method: 'POST',
                headers: {
                    accept: '*/*',
                    'accept-language': 'zh-TW,zh;q=0.9,ja;q=0.8,zh-CN;q=0.7,en-US;q=0.6,en;q=0.5',
                    authorization: 'Bearer ' + token_text,
                    'cache-control': 'no-cache',
                    'content-type': 'application/json',
                    pragma: 'no-cache',
                    'sec-ch-ua': '"Microsoft Edge";v="113", "Chromium";v="113", "Not-A.Brand";v="24"',
                    'sec-ch-ua-mobile': '?0',
                    'sec-ch-ua-platform': '"Windows"',
                    'sec-fetch-dest': 'empty',
                    'sec-fetch-mode': 'cors',
                    'sec-fetch-site': 'cross-site',
                    Referer: 'https://appsumo.com/',
                    'Referrer-Policy': 'strict-origin-when-cross-origin',
                    'User-Agent':
                        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36 Edg/113.0.1774.42',
                },
                body: JSON.stringify([{ Text: text }]),
            }
        );

        if (res.ok) {
            let result = await res.json();
            if (result[0].translations) {
                return result[0].translations[0].text.trim();
            } else {
                throw JSON.stringify(result);
            }
        } else {
            throw `Http Request Error\nHttp Status: ${res.status}\n${await res.text()}`;
        }
    } else {
        throw 'Get Token Failed';
    }
}

export * from './Config';
export * from './info';

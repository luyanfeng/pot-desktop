import { fetch } from '@tauri-apps/plugin-http';

export async function collection(source, target, options = {}) {
    const { config } = options;
    const name = config['name'] ?? 'pot';
    const token = config['token'] ?? '';

    let categoryId = await checkCategory(name, token);
    return await addWordToCategory(categoryId, source, token);
}

async function checkCategory(name, token) {
    let res = await fetch(
        `https://api.frdic.com/api/open/v1/studylist/category?${new URLSearchParams({
            language: 'en',
        })}`,
        {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                Authorization: token,
            },
        }
    );

    let result = await res.json();
    if (result.data) {
        for (let i of result.data) {
            if (i.name === name) {
                return i.id;
            }
        }

        let res1 = await fetch('https://api.frdic.com/api/open/v1/studylist/category', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Authorization: token,
            },
            body: JSON.stringify({
                language: 'en',
                name: name,
            }),
        });
        let result1 = await res1.json();
        if (result1.data) {
            return result1.data.id;
        } else {
            throw 'Create Category Failed';
        }
    } else {
        throw 'Get Category Failed';
    }
}

async function addWordToCategory(id, word, token) {
    let res = await fetch('https://api.frdic.com/api/open/v1/studylist/words', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            Authorization: token,
        },
        body: JSON.stringify({
            id: id,
            language: 'en',
            words: [word],
        }),
    });
    let result = await res.json();
    return result.message;
}

export * from './Config';
export * from './info';

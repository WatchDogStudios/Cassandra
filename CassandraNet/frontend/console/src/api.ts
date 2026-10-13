// Auth-aware API client
// Access token is managed by AuthProvider and passed to this module

let getAccessToken: (() => string | null) | null = null;
let onUnauthorized: (() => void) | null = null;

export function setAuthHandlers(
  tokenGetter: () => string | null,
  unauthorizedHandler: () => void
) {
  getAccessToken = tokenGetter;
  onUnauthorized = unauthorizedHandler;
}

function getHeaders(contentType?: string): Record<string, string> {
  const headers: Record<string, string> = {
    accept: 'application/json',
  };
  
  if (contentType) {
    headers['Content-Type'] = contentType;
  }

  const token = getAccessToken?.();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  
  return headers;
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (res.status === 401) {
    onUnauthorized?.();
    throw new Error('Unauthorized');
  }

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || res.statusText);
  }

  // Handle empty responses
  const contentType = res.headers.get('content-type');
  if (!contentType || !contentType.includes('application/json')) {
    return undefined as T;
  }

  return res.json() as Promise<T>;
}

export async function fetchJson<T>(input: RequestInfo | URL): Promise<T> {
  const res = await fetch(input, { headers: getHeaders() });
  return handleResponse<T>(res);
}

export async function postJson<T>(input: RequestInfo | URL, body: unknown): Promise<T> {
  const res = await fetch(input, {
    method: 'POST',
    headers: getHeaders('application/json'),
    body: JSON.stringify(body),
  });
  return handleResponse<T>(res);
}

export async function putJson<T>(input: RequestInfo | URL, body: unknown): Promise<T> {
  const res = await fetch(input, {
    method: 'PUT',
    headers: getHeaders('application/json'),
    body: JSON.stringify(body),
  });
  return handleResponse<T>(res);
}

export async function deleteJson<T>(input: RequestInfo | URL): Promise<T> {
  const res = await fetch(input, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  return handleResponse<T>(res);
}

export async function uploadFile<T>(
  url: string,
  formData: FormData,
  onProgress?: (percent: number) => void
): Promise<T> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    
    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable && onProgress) {
        const percent = Math.round((e.loaded / e.total) * 100);
        onProgress(percent);
      }
    });

    xhr.addEventListener('load', () => {
      if (xhr.status === 401) {
        onUnauthorized?.();
        reject(new Error('Unauthorized'));
        return;
      }

      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const data = JSON.parse(xhr.responseText);
          resolve(data);
        } catch {
          resolve(undefined as T);
        }
      } else {
        reject(new Error(xhr.responseText || xhr.statusText));
      }
    });

    xhr.addEventListener('error', () => {
      reject(new Error('Network error'));
    });

    xhr.open('POST', url);
    
    const token = getAccessToken?.();
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`);
    }

    xhr.send(formData);
  });
}

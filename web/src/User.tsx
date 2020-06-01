import React, { useEffect, useState } from 'react';
import { Label, TextField, PrimaryButton, Separator, Text } from '@fluentui/react';

function User() {
  const [user, setUser] = useState({} as User);
  const [wechat, setWechat] = useState({} as Wechat);

  useEffect(() => {
    fetch("/wechat")
      .then(res => {
        return res.json();
      }).then((wechat: Wechat) => {
        setWechat(wechat);
      })
  }, [user]);

  useEffect(() => {
    fetch("/user")
      .then(res => {
        if (res.status === 401) {
          window.location.href = res.headers.get("Location") ?? "/";
        } else {
          return res.json();
        }
      }).then((entity: User) => {
        setUser(entity);
      })
  }, []);

  const update = () => {
    fetch('/wechat', {
      method: "PUT",
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(wechat)
    })
      .then(res => {
        if (res.status < 400) {
          alert("Success");
        }
        return res.text();
      }).then(res => {
        console.log(res);
      });
  };
  const onCorpIdChange = (event: React.FormEvent<HTMLInputElement | HTMLTextAreaElement>, newVal?: string) => {
    setWechat({
      ...wechat,
      corp_id: newVal || '',
    });
  }
  const onAgentIdChange = (event: React.FormEvent<HTMLInputElement | HTMLTextAreaElement>, newVal?: string) => {
    setWechat({
      ...wechat,
      agent_id: parseInt(newVal ?? '0'),
    });
  }
  const onSecretChange = (event: React.FormEvent<HTMLInputElement | HTMLTextAreaElement>, newVal?: string) => {
    setWechat({
      ...wechat,
      secret: newVal || '',
    });
  }
  return (
    <div>
      <Label>GitHub 账号</Label> <TextField readOnly value={user.github_login}></TextField>
      <Label>App Key</Label> <TextField readOnly value={user.app_key}></TextField>

      <Label>企业 ID</Label> <TextField onChange={onCorpIdChange} value={wechat.corp_id}></TextField>
      <Label>Agent ID</Label> <TextField onChange={onAgentIdChange} value={wechat.agent_id ? wechat.agent_id.toString() : ""}></TextField>
      <Label>Secret</Label> <TextField onChange={onSecretChange} value={wechat.secret}></TextField>
      <PrimaryButton style={{ marginTop: '10px' }} onClick={update}>更新</PrimaryButton>

      <Separator />
      <Text variant='xLarge'>您的 Callback URL 是: {user.callback_url}</Text>
    </div>
  );
}

interface User {
  github_login: string,
  github_id: number,
  app_key: string,
  callback_url: string,
}

interface Wechat {
  corp_id: string,
  agent_id: number,
  secret: string,
}

export default User;

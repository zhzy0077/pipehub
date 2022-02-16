import React, { useState } from 'react';
import { Stack, Label, TextField, PrimaryButton } from '@fluentui/react';
import { UserEntity } from './User';

interface SendProps {
    user: UserEntity,
}

function Send(props: SendProps) {
    const [message, setMessage] = useState("");
    const send = () => {
        fetch(`${props.user.callback_url}`, {
            method: "POST",
            body: message,
        })
            .then(res => {
                return res.json();
            }).then(entity => {
                const str = JSON.stringify(entity, null, 2);
                alert(str);
            });
    };

    const onMessageUpdate = (event: React.FormEvent<HTMLInputElement | HTMLTextAreaElement>, newVal?: string) => {
        setMessage(newVal || "");
    }

    return (
        <Stack tokens={{ childrenGap: '15px' }}>
            <Stack.Item>
                <Label>Payload</Label>
            </Stack.Item>
            <Stack.Item>
                <TextField multiline rows={3} onChange={onMessageUpdate} value={message} />
            </Stack.Item>
            <Stack.Item>
                <PrimaryButton onClick={send}>测试发送</PrimaryButton>
            </Stack.Item>
        </Stack>
    );
}

export default Send;

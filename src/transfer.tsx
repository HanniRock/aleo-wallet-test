/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/12 10:45
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */

import React, {useEffect, useState} from 'react';
import init, {transfer} from "wasm-lib";

const Transfer = () => {
    const [privateKey, setPrivateKey] = useState('');
    const [record, setRecord] = useState('');
    const [fee_record, setFeeRecord] = useState<string | undefined>(undefined);
    const [amount, setAmount] = useState(0);
    const [fee, setFee] = useState<number | undefined>(undefined);
    const [recipient, setRecipient] = useState('');
    const [broadcast, setBroadcast] = useState('');

    useEffect(() => {
        init();
    }, []);

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        console.log({
            private_key: privateKey,
            record: record,
            fee_record: fee_record || undefined,
            amount: amount,
            fee: fee || undefined,
            recipient: recipient,
            broadcast: broadcast,
        });
        try {
            const records = transfer(privateKey, record, fee_record, BigInt(amount), BigInt(fee!), recipient, broadcast);
            console.log(records);
        } catch (error) {
            console.error("Failed to request records:", error);
        }

    };

    const handleInputChange = <T, >(
        event: React.ChangeEvent<HTMLInputElement>,
        setState: React.Dispatch<React.SetStateAction<T>>
    ) => {
        const value = event.target.value;
        if (event.target.type === 'number') {
            setState((value === '' ? undefined : Number(value)) as T);
        } else {
            setState(value as T);
        }
    };


    return (
        <div>
            <h1>请输入Transfer参数</h1>
            <form onSubmit={handleSubmit}>
                <label htmlFor="private_key">Private Key:</label>
                <input
                    type="text"
                    id="private_key"
                    name="private_key"
                    placeholder="请输入PrivateKey"
                    value={privateKey}
                    onChange={(e) => handleInputChange<string>(e, setPrivateKey)}
                />
                <br/>
                <br/>

                <label htmlFor="record">Record:</label>
                <input
                    type="text"
                    id="record"
                    name="record"
                    placeholder="请输入record"
                    required
                    value={record}
                    onChange={(e) => handleInputChange<string>(e, setRecord)}
                />
                <br/>
                <br/>

                <label htmlFor="fee_record">FeeRecord (可选):</label>
                <input
                    type="text"
                    id="fee_record"
                    name="fee_record"
                    placeholder="请输入fee_record"
                    value={fee_record === undefined ? '' : fee_record}
                    onChange={(e) => handleInputChange<string | undefined>(e, setFeeRecord)}
                />
                <br/>
                <br/>

                <label htmlFor="amount">Amount:</label>
                <input
                    type="number"
                    id="amount"
                    name="amount"
                    placeholder="请输入数量"
                    value={amount}
                    onChange={(e) => handleInputChange<number>(e, setAmount)}
                />
                <br/>
                <br/>

                <label htmlFor="fee">Fee (可选):</label>
                <input
                    type="number"
                    id="fee"
                    name="fee"
                    placeholder="请输入fee数量"
                    value={fee === undefined ? '' : fee}
                    onChange={(e) => handleInputChange<number | undefined>(e, setFee)}
                />
                <br/>
                <br/>

                <label htmlFor="recipient">Recipient:</label>
                <input
                    type="text"
                    id="recipient"
                    name="recipient"
                    placeholder="请输入Recipient"
                    required
                    value={recipient}
                    onChange={(e) => handleInputChange<string>(e, setRecipient)}
                />
                <br/>
                <br/>

                <label htmlFor="broadcast">Broadcast:</label>
                <input
                    type="text"
                    id="broadcast"
                    name="broadcast"
                    placeholder="请输入Broadcast"
                    required
                    value={broadcast}
                    onChange={(e) => handleInputChange<string>(e, setBroadcast)}
                />
                <br/>
                <br/>

                <input type="submit" value="Submit"/>
            </form>
        </div>
    );
};

export default Transfer;

import React, { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';

import { addSSHKey } from '../api/api.js';
import { getErrorMessage } from '../utils/errorUtils';
import { Button } from './ui/Button';

export const AddSSHKeyForm = ({ onSuccess, tenantId: tenant }) => {
    const [name, setName] = useState('');
    const [ssh_pub_key, setSshPubKey] = useState('');
    const queryClient = useQueryClient();

    const { mutate, isLoading: isAdding } = useMutation({
        mutationFn: addSSHKey,
        onSuccess: () => {
             alert("SSH Key Added!");
             queryClient.invalidateQueries({ queryKey: ['sshKeys'] });
             setName(''); setSshPubKey('');
             if(onSuccess) onSuccess();
        },
        onError: (error) => alert(`Error adding SSH Key: ${getErrorMessage(error)}`),
    });

    const handleSubmit = (e) => {
        e.preventDefault();
        if (!name || !ssh_pub_key) { alert("Please provide a name and the public key."); return; }
        mutate({ name, ssh_pub_key, tenant });
    };

    return (
         <form onSubmit={handleSubmit} className="p-4 border border-dashed border-gray-300 rounded-md mt-4 bg-gray-50 space-y-3">
             <h4 className="text-sm font-medium text-gray-700 mb-2">Add New SSH Key</h4>
             <div>
                <label htmlFor="ssh-name" className="block text-xs font-medium text-gray-600 mb-1">Key Name</label>
                <input id="ssh-name" type="text" value={name} onChange={(e) => setName(e.target.value)} required className="input-class" disabled={isAdding} />
            </div>
            <div>
                 <label htmlFor="ssh-public-key" className="block text-xs font-medium text-gray-600 mb-1">Public Key</label>
                 <textarea id="ssh-public-key" value={ssh_pub_key} onChange={(e) => setSshPubKey(e.target.value)} required rows={3} placeholder="ssh-rsa AAAAB3NzaC1yc2EAAA..." className="input-class font-mono" disabled={isAdding}></textarea>
            </div>
             <Button type="submit" size="sm" disabled={isAdding}>
                 {isAdding ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                 {isAdding ? 'Adding...' : 'Add SSH Key'}
             </Button>
         </form>
    );
};
import React, { useState } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';

import { addVpc } from '../api/api.js';
import { getErrorMessage } from '../utils/errorUtils';
import { Button } from './ui/Button';
import { CheckCircle } from 'lucide-react';

export const AddVpcForm = ({ onSuccess, tenantId: tenant }) => {
    const [name, setName] = useState('');
    const [cidr, setCidr] = useState('');
    const [nat, setNat] = useState(false);
    const queryClient = useQueryClient();

    const { mutate, isLoading: isAdding } = useMutation({
        mutationFn: addVpc,
        onSuccess: () => {
            alert("VPC Added!")
            queryClient.invalidateQueries({ queryKey: ['vpcs'] });
            setName(''); setCidr('');
            if(onSuccess) onSuccess();
        },
        onError: (error) => alert(`Error adding VPC: ${getErrorMessage(error)}`),
    });

    const handleSubmit = (e) => {
        e.preventDefault();
        if (!name || !cidr) { alert("Please provide a VPC name and CIDR"); return; }
        mutate({ name, cidr, nat, tenant });
    };

    return (
         <form onSubmit={handleSubmit} className="p-4 border border-dashed border-gray-300 rounded-md mt-4 bg-gray-50 space-y-3">
            <h4 className="text-sm font-medium text-gray-700 mb-2">Add New VPC</h4>
            <div>
                <label htmlFor="name" className="block text-xs font-medium text-gray-600 mb-1">VPC Name</label>
                <input id="name" type="text" value={name} onChange={(e) => setName(e.target.value)} required className="input-class" disabled={isAdding} />
            </div>
            <div>
                 <label htmlFor="cidr" className="block text-xs font-medium text-gray-600 mb-1">VPC Subnet</label>
                 <textarea id="cidr" value={cidr} onChange={(e) => setCidr(e.target.value)} required placeholder="172.16.1.0/24" className="input-class font-mono" disabled={isAdding}></textarea>
            </div>

            <div>
                <label htmlFor="nat" className="block text-xs font-medium text-gray-600 mb-1">VPC Subnet</label>
                <select id="nat" value={nat} onChange={(e) => setNat(e.target.value)} className="input-class">
                    <option value={false}>No</option>
                    <option value={true}>Yes</option>
                </select>
            </div>

            <Button type="submit" size="sm" disabled={isAdding}>
                {isAdding ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                {isAdding ? 'Adding...' : 'Add VPC'}
            </Button>
         </form>
    );
};
import React, { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';

import { addTenant } from '../api/api.js';
import { getErrorMessage } from '../utils/errorUtils';
import { Button } from './ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from './ui/Card';

export const AddTenantForm = ({ onSuccess, onCancel }) => {
    const [name, setName] = useState('');
    const queryClient = useQueryClient();

    const { mutate, isLoading: isAdding } = useMutation({
        mutationFn: addTenant,
        onSuccess: () => {
             alert("Tenant has been Added!");
             queryClient.invalidateQueries({ queryKey: ['tenants'] });
             setName('');
             if(onSuccess) onSuccess();
        },
        onError: (error) => alert(`Error adding Tenant: ${getErrorMessage(error)}`),
    });

    const handleSubmit = (e) => {
        e.preventDefault();
        if (!name) { alert("Please provide a Tenant name"); return; }
        mutate({ name });
    };

    return (
        <Card className="mb-6 max-w-md">
            <CardHeader>
                <CardTitle>Add New Tenant</CardTitle>
            </CardHeader>
            <CardContent>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label htmlFor="tenant-name" className="block text-sm font-medium text-gray-700">Tenant Name</label>
                        <input 
                            type="text" 
                            id="tenant-name" 
                            className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2"
                            placeholder="compute-project"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            required
                        />
                    </div>
                    <div className="flex space-x-2">
                        <Button type="button" onClick={onCancel}>Cancel</Button>
                        <Button type="submit" variant="default" disabled={isAdding}>
                            {isAdding ? 'Creating...' : 'Create Tenant'}
                        </Button>
                    </div>
                </form>
            </CardContent>
        </Card>
    );
};
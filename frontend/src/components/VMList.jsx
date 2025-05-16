import React, { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Trash2, Loader2 } from 'lucide-react';

import { deleteVM } from '../api/api.js';
import { getErrorMessage } from '../utils/errorUtils';
import { Button } from './ui/Button';
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from './ui/Table';

export const VMList = ({ vms = [], tenantName }) => {
    const [deletingName, setDeletingName] = useState(null);
    const [showSuccess, setShowSuccess] = useState(false);

    const queryClient = useQueryClient();

    const { mutate: performDelete, isLoading: isDeleting } = useMutation({
        mutationFn: deleteVM,
        onSuccess: (data, deleteObject) => {
            setShowSuccess(true);
            console.log(`VM ${deleteObject.name} deleted successfully!`);
            queryClient.invalidateQueries({ queryKey: ['vms', tenantName] });
            setDeletingName(null);
        },
        onError: (error, deleteObject) => {
            console.log(`VM ${deleteObject.name} deletion failed: ${getErrorMessage(error)}`);
            setDeletingId(null);
        },
    });

    const handleDeleteClick = (vmName) => {
        if (deletingName) return;
        if (window.confirm(`Are you sure you want to delete VM ${vmName}?`)) {
            setDeletingName(vmName);
            performDelete({ name: vmName, tenant: tenantName });
        }
    };

    if (vms.length === 0) {
        return <p className="text-sm text-gray-500">No VMs found for this tenant.</p>;
    }

    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Memory</TableHead>
                    <TableHead>CPU</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>OS</TableHead>
                    <TableHead>Disk</TableHead>
                    <TableHead>IP Addresses</TableHead>
                    <TableHead>Actions</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {vms.map(vm => {
                    const isDeletingThisVm = deletingName === vm.name;
                    return (
                        <TableRow key={vm.name}>
                            <TableCell className="font-medium text-gray-900">{vm.name ?? 'N/A'}</TableCell>
                            <TableCell>{vm.ram ?? 'N/A'}</TableCell>
                            <TableCell>{vm.cpu ?? 'N/A'}</TableCell>
                            <TableCell>
                                <span className={`status-badge ${vm.state === 'running' ? 'status-running' : vm.state === 'stopped' ? 'status-stopped' : 'status-other'}`}>
                                    {vm.state ?? 'unknown'}
                                </span>
                            </TableCell>
                            <TableCell>{vm.os ?? 'N/A'}</TableCell>
                            <TableCell>{vm.disk_size != null ? `${vm.disk_size}GB` : 'N/A'}</TableCell>
                            <TableCell>{vm.ip_addresses.map(ip => (
                                <span key={ip} className="block text-sm text-gray-500">{ip}</span>
                            )) ?? 'N/A'
                            }</TableCell>
                            <TableCell>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className="text-red-600 hover:text-red-800 hover:bg-red-50 w-8 h-8 p-0"
                                    onClick={() => handleDeleteClick(vm.name)}
                                    disabled={isDeletingThisVm || !!deletingName}
                                    title={isDeletingThisVm ? "Deleting..." : `Delete VM ${vm.name}`}
                                >
                                    {isDeletingThisVm ? <Loader2 className="h-4 w-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                                    {showSuccess && <span className="text-green-600">VM  Deleted!</span>}
                                </Button>
                            </TableCell>
                        </TableRow>
                    );
                 })}
            </TableBody>
        </Table>
    );
};

// Add CSS classes for status badges if not using a UI library
// .status-badge { padding: 0.125rem 0.5rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 500; }
// .status-running { background-color: #DCFCE7; color: #166534; } /* Green */
// .status-stopped { background-color: #FEE2E2; color: #991B1B; } /* Red */
// .status-other { background-color: #FEF9C3; color: #854D0E; } /* Yellow */
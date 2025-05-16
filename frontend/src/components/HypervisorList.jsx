import React from 'react';
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from './ui/Table';

export const HypervisorList = ({ hypervisors = [] }) => {
    if (hypervisors.length === 0) {
        return <p className="text-sm text-gray-500">No hypervisors found.</p>;
    }
     return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Hostname</TableHead>
                    <TableHead>Total Memory</TableHead>
                    <TableHead>Total CPUs</TableHead>
                    <TableHead>Used Memory</TableHead>
                    <TableHead>Used CPUs</TableHead>
                    <TableHead>Total VMs</TableHead>
                    <TableHead>Architecture</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {hypervisors.map(hypervisor => (
                    <TableRow key={hypervisor.hostname}>
                        <TableCell className="font-medium text-gray-900">{hypervisor.hostname ?? 'N/A'}</TableCell>
                        <TableCell>{hypervisor.total_ram ?? 'N/A'}</TableCell>
                        <TableCell>{hypervisor.total_cpu ?? 'N/A'}</TableCell>
                        <TableCell>{hypervisor.used_ram ?? 'N/A'}</TableCell>
                        <TableCell>{hypervisor.used_cpu ?? 'N/A'}</TableCell>
                        <TableCell>{hypervisor.hosted_vms ?? 'N/A'}</TableCell>
                        <TableCell>{hypervisor.arch ?? 'N/A'}</TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    );
};
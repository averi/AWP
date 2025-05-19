// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from './ui/Table';

export const ProviderNetworkList = ({ networks = [] }) => {
    if (networks.length === 0) {
        return <p className="text-sm text-gray-500">No provider networks found.</p>;
    }
     return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>VLAN ID</TableHead>
                    <TableHead>Subnet</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {networks.map(net => (
                    <TableRow key={net.name}>
                        <TableCell className="font-medium text-gray-900">{net.name ?? 'N/A'}</TableCell>
                        <TableCell>{net.vlan ?? 'N/A'}</TableCell>
                        <TableCell>{net.subnet ?? 'N/A'}</TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    );
};
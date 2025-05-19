// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';

import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from './ui/Table';
import { Button } from './ui/Button';
import { Trash2, Loader2 } from 'lucide-react';

export const VpcTable = ({ vpcs = [], onRemove, removingVpcName }) => {

    const handleRemoveClick = (vpcName, vpcId, removingVpcName) => {
        if (removingVpcName) return;
        if (window.confirm(`Are you sure you want to remove VPC "${vpcName}"?`)) {
            onRemove(vpcId);
        }
    };

    if (vpcs.length === 0) { return <p className="text-sm text-gray-500">No VPCs found.</p>; }
    
    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>CIDR</TableHead>
                    <TableHead>NAT</TableHead>
                    <TableHead>Actions</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {vpcs.map(vpc => {
                    const isRemovingVpc = removingVpcName === vpc.name;

                    return (
                        <TableRow key={vpc.name}>
                            <TableCell className="font-medium text-gray-900">{vpc.name ?? 'N/A'}</TableCell>
                            <TableCell>{vpc.cidr ?? 'N/A'}</TableCell>
                            <TableCell>{vpc.nat ? 'Yes' : 'No'}</TableCell>
                            <TableCell>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className="text-red-600 hover:text-red-800 hover:bg-red-50 w-8 h-8 p-0"
                                    onClick={() => handleRemoveClick(vpc.name, vpc.id)}
                                    disabled={isRemovingVpc || !!removingVpcName}
                                    title={isRemovingVpc ? "Removing..." : `Remove VPC ${vpc.name}`}
                                >
                                    {isRemovingVpc ? <Loader2 className="h-4 w-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                                </Button>
                            </TableCell>
                        </TableRow>
                    );
                })}
            </TableBody>
        </Table>
    );
};
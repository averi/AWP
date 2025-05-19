// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';
import { Trash2, Loader2 } from 'lucide-react';

import { Button } from './ui/Button';
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from './ui/Table';

export const SSHKeyList = ({ sshKeys = [], onRemove, removingKeyName }) => {

    const handleRemoveClick = (keyName) => {
        if (removingKeyName) return;
        if (window.confirm(`Are you sure you want to remove SSH Key "${keyName}"?`)) {
            onRemove(keyName);
        }
    };

    if (sshKeys.length === 0) {
        return <p className="text-sm text-gray-500">No SSH keys found.</p>;
    }

    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Fingerprint</TableHead>
                    <TableHead>Actions</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {sshKeys.map(key => {
                    const isRemovingSSHKey = removingKeyName === key.name;
                    return (
                        <TableRow key={key.name}>
                            <TableCell className="font-medium text-gray-900">{key.name ?? 'N/A'}</TableCell>
                            <TableCell className="font-mono text-xs">{key.fingerprint ?? 'N/A'}</TableCell>
                            <TableCell>
                                <Button
                                    variant="ghost" size="sm"
                                    className="text-red-600 hover:text-red-800 hover:bg-red-50 w-8 h-8 p-0"
                                    onClick={() => handleRemoveClick(key.name)}
                                    disabled={isRemovingSSHKey || !!removingKeyName}
                                    title={isRemovingSSHKey ? "Removing..." : `Remove SSH Key`}
                                >
                                   {isRemovingSSHKey ? <Loader2 className="h-4 w-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                                </Button>
                            </TableCell>
                        </TableRow>
                    );
                })}
            </TableBody>
        </Table>
    );
};
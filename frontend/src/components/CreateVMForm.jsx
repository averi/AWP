import React, { useState, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Loader2, CheckCircle, XCircle  } from 'lucide-react';

import { createVM, listObjects, listProviderNetworks } from '../api/api.js';
import { getErrorMessage } from '../utils/errorUtils';
import { Button } from './ui/Button';

const inputClass = "block w-full px-3 py-1.5 text-sm border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed";
const selectClass = `${inputClass} appearance-none`;

const defaultValues = {
    ram: 1,
    cpu: 1,
    disk_size: 10,
    arch: 'x86_64',
    networkingMode: 'l2-tenant',
};

export const CreateVMForm = ({ onSuccess, tenantId, tenantName }) => {
    const [name, setName] = useState('');
    const [ram, setRam] = useState(defaultValues.ram);
    const [cpu, setCpu] = useState(defaultValues.cpu);
    const [os, setOs] = useState('');
    const [diskSize, setDiskSize] = useState(defaultValues.disk_size);
    const [vpc, setVpc] = useState('');
    const [sshPubKey, setSshPubKey] = useState('');
    const [arch, setArch] = useState(defaultValues.arch);
    const [networkingMode, setNetworkingMode] = useState(defaultValues.networkingMode);
    const [providerNetwork, setProviderNetwork] = useState('');
    const [submissionStatus, setSubmissionStatus] = useState('form');
    const [submissionError, setSubmissionError] = useState(null);

    const osOptions = [
      { value: 'rhel9', label: 'RHEL 9' },
      { value: 'fedora42', label: 'Fedora 42' },
  ];

    const queryClient = useQueryClient();

    const { data: vpcsData = [], isLoading: isLoadingVpcs, isError: isErrorVpcs, error: errorVpcs, isFetching: isFetchingVpcs } = useQuery({ queryKey: ["vpcsData", tenantId], queryFn: ({ queryKey }) => listObjects("vpcs", queryKey[1]) });
    const { data: sshKeysData = [], isLoading: isLoadingSSHKeys, isError: isErrorSSHKeys, error: errorSSHKeys, isFetching: isFetchingSSHKeys } = useQuery({ queryKey: ["sshKeysData", tenantId], queryFn: ({ queryKey }) => listObjects("ssh_pub_keys", queryKey[1]) });
    const { data: providerNetworksData = [], isLoading: isLoadingProviderNetworks, isError: isErrorProviderNetworks, error: errorProviderNetworks, isFetching: isFetchingProviderNetworks } = useQuery({ queryKey: ["providernetworks"], queryFn: listProviderNetworks });

    const { mutate, isLoading: isCreating } = useMutation({
        mutationFn: createVM,
        onSuccess: (data) => {
            console.log("VM creation successful:", data);
            queryClient.invalidateQueries({ queryKey: ['vms', tenantName] });
            setName('');
            setRam(defaultValues.ram);
            setCpu(defaultValues.cpu);
            setOs('');
            setDiskSize(defaultValues.disk_size);
            setVpc('');
            setSshPubKey('');
            setArch(defaultValues.arch);
            setNetworkingMode(defaultValues.networkingMode);
            setProviderNetwork('');
            setSubmissionStatus('success');
            if (onSuccess) onSuccess(data);
        },
        onError: (error) => {
            console.error("VM creation failed:", error);
            setSubmissionError(error);
            setSubmissionStatus('error');
            alert(`Error creating VM: ${getErrorMessage(error)}`);
        },
    });

    const handleSubmit = (event) => {
        event.preventDefault();
        const isBridgedButNoNetwork = networkingMode === 'l2-bridged' && !providerNetwork;
        if (!name || !ram || !cpu || !os || !diskSize || !vpc || !sshPubKey || !tenantName || !arch || !networkingMode || isBridgedButNoNetwork) {
            alert("Please fill in all required VM details, including Networking Mode and Provider Network if applicable.");
            return;
        }

        const vmData = {
            name,
            ram: parseInt(ram, 10),
            cpu: parseInt(cpu, 10),
            os,
            disk_size: parseInt(diskSize, 10),
            vpc,
            ssh_pub_key: sshPubKey,
            tenant: tenantName,
            arch,
            networking: networkingMode,
        };

        if (networkingMode === 'l2-bridged') {
            vmData.network = providerNetwork;
        }

        console.log("Submitting VM Data:", vmData);
        mutate(vmData);
    };

    const handleResetForm = () => {
        setSubmissionStatus('form');
        setSubmissionError(null);
    };

    const isSubmitDisabled = isCreating || isLoadingVpcs || isLoadingSSHKeys || isLoadingProviderNetworks || !name || !vpc || !sshPubKey || !ram || !cpu || !os || !diskSize || !arch || !networkingMode || (networkingMode === 'l2-bridged' && !providerNetwork);
    const isFormDisabled = isCreating || isLoadingVpcs || isLoadingSSHKeys;

    return (
        <div className="mt-4">
            {submissionStatus === 'success' && (
                <div className="p-4 text-center text-green-700 bg-green-100 border border-green-300 rounded-md">
                    <CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-2" />
                    <p className="font-semibold">VM Created Successfully!</p>
                    <Button size="sm" variant="outline" onClick={handleResetForm} className="mt-4">
                       Create Another VM
                    </Button>
                </div>
            )}

            {submissionStatus === 'error' && (
                 <div className="p-4 text-center text-red-700 bg-red-100 border border-red-300 rounded-md">
                    <XCircle className="mx-auto h-12 w-12 text-red-500 mb-2" />
                    <p className="font-semibold">VM Creation Failed</p>
                    <p className="text-sm mt-1">{getErrorMessage(submissionError)}</p>
                    <Button size="sm" variant="outline" onClick={handleResetForm} className="mt-4">
                        Try Again
                    </Button>
                </div>
            )}

            {submissionStatus === 'form' && (
                <form onSubmit={handleSubmit} className="p-4 border border-dashed border-gray-300 rounded-md bg-gray-50 space-y-3">
                <h4 className="text-lg font-semibold text-gray-800 mb-3">Create New Virtual Machine</h4>

                {/* Name */}
                <div>
                    <label htmlFor="vm-name" className="block text-xs font-medium text-gray-600 mb-1">VM Name *</label>
                    <input id="vm-name" type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g., my-web-server-prod" required className={inputClass} disabled={isFormDisabled} />
                </div>

                {/* OS */}
                <div>
                <label htmlFor="vm-os" className="block text-xs font-medium text-gray-600 mb-1">Operating System *</label>
                <select id="vm-os" value={os} onChange={(e) => setOs(e.target.value)} required className={selectClass || inputClass} disabled={isFormDisabled}>
                <option value="" disabled>Select Operating System...</option>
                    {osOptions.map((option) => (
                        <option key={option.value} value={option.value}>
                            {option.label}
                        </option>
                    ))}
                </select>
                </div>

                {/* VPC Dropdown */}
                <div>
                    <label htmlFor="vm-vpc" className="block text-xs font-medium text-gray-600 mb-1">VPC / Network *</label>
                    <select
                        id="vm-vpc"
                        value={vpc}
                        onChange={(e) => setVpc(e.target.value)}
                        required
                        className={selectClass}
                        disabled={isFormDisabled || !vpcsData || vpcsData.length === 0}
                    >
                        <option value="" disabled>
                            {isLoadingVpcs ? "Loading VPCs..." : isErrorVpcs ? "Error loading VPCs" : !vpcsData ? "Loading..." : "Select VPC"}
                        </option>
                        {isErrorVpcs && <option disabled>Error: {getErrorMessage(errorVpcs)}</option>}
                        {vpcsData && vpcsData.map((VpcObject) => (
                            <option key={VpcObject.name} value={VpcObject.name}>{VpcObject.name}</option>
                        ))}
                    </select>
                    {isLoadingVpcs && <Loader2 className="inline-block ml-2 h-4 w-4 animate-spin" />}
                </div>

                {/* Networking Mode */}
                <div>
                    <label htmlFor="vm-networking-mode" className="block text-xs font-medium text-gray-600 mb-1">Networking Mode *</label>
                    <select
                        id="vm-networking-mode"
                        value={networkingMode}
                        onChange={(e) => {
                            setNetworkingMode(e.target.value);
                            if (e.target.value !== 'l2-bridged') {
                                setProviderNetwork('');
                            }
                        }}
                        required
                        className={selectClass}
                        disabled={isFormDisabled}
                    >
                        <option value="l2-tenant">L2 Tenant</option>
                        <option value="l2-bridged">L2 Bridged</option>
                    </select>
                </div>

                {/* Provider Network Dropdown (Conditional) */}
                {networkingMode === 'l2-bridged' && (
                    <div>
                        <label htmlFor="vm-provider-network" className="block text-xs font-medium text-gray-600 mb-1">Provider Network *</label>
                        <select id="vm-provider-network" value={providerNetwork} onChange={(e) => setProviderNetwork(e.target.value)} required className={selectClass} disabled={isFormDisabled || isLoadingProviderNetworks || !providerNetworksData || providerNetworksData.length === 0}>
                            <option value="" disabled>{isLoadingProviderNetworks ? "Loading Networks..." : isErrorProviderNetworks ? "Error loading Networks" : !providerNetworksData || providerNetworksData.length === 0 ? "No provider networks found" : "Select Provider Network"}</option>
                            {isErrorProviderNetworks && <option disabled>Error: {getErrorMessage(errorProviderNetworks)}</option>}
                            {providerNetworksData && providerNetworksData.map((network) => (<option key={network.name} value={network.name}>{network.name}</option>))}
                        </select>
                        {isLoadingProviderNetworks && <Loader2 className="inline-block ml-2 h-4 w-4 animate-spin" />}
                    </div>
                )}

                {/* SSH Key Dropdown */}
                <div>
                    <label htmlFor="vm-sshkey" className="block text-xs font-medium text-gray-600 mb-1">SSH Public Key *</label>
                    <select
                        id="vm-sshkey"
                        value={sshPubKey}
                        onChange={(e) => setSshPubKey(e.target.value)}
                        required
                        className={selectClass}
                        disabled={isFormDisabled || !sshKeysData || sshKeysData.length === 0}
                    >
                        <option value="" disabled>
                            {isLoadingSSHKeys ? "Loading SSH keys..." : isErrorSSHKeys ? "Error loading SSH Keys" : !sshKeysData ? "Loading..." : "Select SSH Pub key"}
                        </option>
                        {isErrorSSHKeys && <option disabled>Error: {getErrorMessage(errorSSHKeys)}</option>}
                        {sshKeysData && sshKeysData.map((keyObj) => (
                            <option key={keyObj.name} value={keyObj.name}>
                                {keyObj.name}
                            </option>
                        ))}
                    </select>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                    {/* RAM */}
                    <div>
                        <label htmlFor="vm-ram" className="block text-xs font-medium text-gray-600 mb-1">RAM (GB)</label>
                        <input id="vm-ram" type="number" value={ram} onChange={(e) => setRam(e.target.value)} min="1" required className={inputClass} disabled={isFormDisabled} />
                    </div>
                    {/* CPU */}
                    <div>
                        <label htmlFor="vm-cpu" className="block text-xs font-medium text-gray-600 mb-1">CPU Cores</label>
                        <input id="vm-cpu" type="number" value={cpu} onChange={(e) => setCpu(e.target.value)} min="1" required className={inputClass} disabled={isFormDisabled} />
                    </div>
                    {/* Disk Size */}
                    <div>
                        <label htmlFor="vm-disk" className="block text-xs font-medium text-gray-600 mb-1">Disk Size (GB)</label>
                        <input id="vm-disk" type="number" value={diskSize} onChange={(e) => setDiskSize(e.target.value)} min="10" required className={inputClass} disabled={isFormDisabled} />
                    </div>
                </div>

                {/* Architecture */}
                <div>
                    <label htmlFor="vm-arch" className="block text-xs font-medium text-gray-600 mb-1">Architecture *</label>
                    <select id="vm-arch" value={arch} onChange={(e) => setArch(e.target.value)} required className={selectClass} disabled={isFormDisabled}>
                        <option value="x86_64">x86_64 (Intel/AMD)</option>
                        <option value="aarch64">aarch64 (ARM)</option>
                    </select>
                </div>

                <Button type="submit" size="sm" disabled={isSubmitDisabled}>
                    {isCreating ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                    {isCreating ? 'Creating VM...' : 'Create VM'}
                </Button>
                </form>
             )}
        </div>
    );
};
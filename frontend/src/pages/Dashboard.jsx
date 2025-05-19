// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React, { useState } from 'react';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Network, Server, KeyRound, Globe, Building, PlusCircle, MinusCircle } from 'lucide-react';

import { listTenants, listObjects, listProviderNetworks, listHypervisors, removeSSHKey, removeVpc, removeTenant } from '../api/api.js';

import { Card, CardHeader, CardTitle, CardContent } from '../components/ui/Card';
import { Select } from '../components/ui/Select';
import { Button } from '../components/ui/Button';
import { Skeleton } from '../components/ui/Skeleton';
import { ErrorMessage } from '../components/ui/ErrorMessage';

import { CreateVMForm } from '../components/CreateVMForm';
import { VMList } from '../components/VMList';
import { AddSSHKeyForm } from '../components/AddSSHKeyForm';
import { SSHKeyList } from '../components/SSHKeyList';
import { ProviderNetworkList } from '../components/ProviderNetworkList';
import { VpcTable } from '../components/VpcTable';
import { HypervisorList } from '../components/HypervisorList';
import { AddVpcForm } from '../components/AddVpcForm';
import { AddTenantForm } from '../components/AddTenantForm.jsx';

import { getErrorMessage } from '../utils/errorUtils';

export function Dashboard() {
    const queryClient = useQueryClient();

    const [selectedTenantId, setSelectedTenantId] = useState('');
    const [showAddKeyForm, setShowAddKeyForm] = useState(false);
    const [showCreateForm, setShowCreateForm] = useState(false);
    const [showAddVpcForm, setShowAddVpcForm] = useState(false);
    const [showAddTenantForm, setShowAddTenantForm] = useState(false);  

    const { data: tenants = [], isLoading: isLoadingTenants, isError: isErrorTenants, error: errorTenants, isFetching: isFetchingTenants } = useQuery({ queryKey: ["tenants"], queryFn: listTenants });
    const { data: vpcs = [], isLoading: isLoadingVpcs, isError: isErrorVpcs, error: errorVpcs, isFetching: isFetchingVpcs } = useQuery({ queryKey: ["vpcs", selectedTenantId], queryFn: ({ queryKey }) => listObjects("vpcs", queryKey[1]), enabled: !!selectedTenantId });
    const { data: vms = [], isLoading: isLoadingVms, isError: isErrorVms, error: errorVms, isFetching: isFetchingVms } = useQuery({ queryKey: ["vms", selectedTenantId], queryFn: ({ queryKey }) => listObjects("virtualmachines", queryKey[1]), enabled: !!selectedTenantId });
    const { data: sshKeys = [], isLoading: isLoadingSSHKeys, isError: isErrorSSHKeys, error: errorSSHKeys, isFetching: isFetchingSSHKeys } = useQuery({ queryKey: ["sshKeys",selectedTenantId], queryFn: ({ queryKey }) => listObjects("ssh_pub_keys", queryKey[1]), enabled: !!selectedTenantId });
    const { data: providerNetworks = [], isLoading: isLoadingProviderNets, isError: isErrorProviderNets, error: errorProviderNets, isFetching: isFetchingProviderNets } = useQuery({ queryKey: ["providerNetworks"], queryFn: listProviderNetworks });
    const { data: hypervisors = [], isLoading: isLoadingHypervisors, isError: isErrorHypervisors, error: errorHypervisors, isFetching: isFetchingHypervisors } = useQuery({ queryKey: ["Hypervisors"], queryFn: listHypervisors });

    const { mutate: removeSshKeyMutate, isLoading: isRemovingSshKey, variables: removingSSHVariables } = useMutation({
        mutationFn: removeSSHKey,
        onSuccess: (data, variables) => {
            alert(`SSH Key ${variables.name} has been removed successfully!`);
            queryClient.invalidateQueries({ queryKey: ['sshKeys'] });
        },
        onError: (error, variables) => {
            alert(`Error removing SSH Key ${variables.name}: ${getErrorMessage(error)}`);
        },
    });

    const { mutate: removeVpcMutate, isLoading: isRemovingVpc, variables: removingVpcVariables } = useMutation({
        mutationFn: removeVpc,
        onSuccess: (data, variables) => {
            alert(`VPC ${variables.id} has been removed successfully!`);
            queryClient.invalidateQueries({ queryKey: ['vpcs'] });
        },
        onError: (error, variables) => {
            alert(`Error removing VPC ${variables.id}: ${getErrorMessage(error)}`);
        },
    });

    const { mutate: removeTenantMutate, isLoading: isRemovingTenant } = useMutation({
        mutationFn: removeTenant,
        onSuccess: (data, variables) => {
            alert(`Tenant ${variables.name || variables.id} has been removed successfully!`);
            queryClient.invalidateQueries({ queryKey: ['tenants'] });
            setSelectedTenantId('');
        },
        onError: (error, variables) => {
            alert(`Error removing Tenant ${variables.name || variables.id}: ${getErrorMessage(error)}`);
        },
    });

    const handleDeleteTenant = () => {
        if (!selectedTenantId) return;

        const tenantToDelete = tenants.find(t => t.id === selectedTenantId);
        const tenantName = tenantToDelete?.name || `Tenant ${selectedTenantId}`;

        if (window.confirm(`Are you sure you want to delete ${tenantName}? This action cannot be undone.`)) {
            removeTenantMutate({ id: selectedTenantId, name: tenantToDelete?.name });
        }
    };

    const handleFormClose = () => {
        setShowCreateForm(false);
    };

    const handleTenantChange = (event) => { setSelectedTenantId(event.target.value); };
    const handleRemoveSshKey = (keyName) => { removeSshKeyMutate({ name: keyName, id: selectedTenantId }); };
    const handleRemoveVpc = (vpcId) => { removeVpcMutate({ id: vpcId, tenant: selectedTenantId }); };

    const handleAddNewTenant = () => {
        setShowAddTenantForm(true);
    };

    const tenantOptions = tenants.map(tenant => ({ value: tenant.id, label: tenant.name ?? `Tenant ${tenant.id}` }));
    const selectedTenantName = tenants.find(t => t.id === selectedTenantId)?.name || '';

    return (
        <div className="min-h-screen bg-gray-100 p-4 md:p-8 font-sans">
            <div className="mb-8 max-w-md">
                <label htmlFor="tenant-select" className="block text-sm font-medium text-gray-700 mb-1"> Select Tenant {isFetchingTenants ? '(Refreshing...)' : ''} </label>
                {isLoadingTenants ? <Skeleton className="h-10 w-full" /> : isErrorTenants ? <ErrorMessage message={`Error loading tenants: ${getErrorMessage(errorTenants)}`} /> : (
                    <div className="inline-grid w-full grid-cols-[1fr_auto] gap-x-2 gap-y-2 items-start">
                        <Select
                            id="tenant-select"
                            value={selectedTenantId}
                            onChange={handleTenantChange}
                            options={tenantOptions}
                            placeholder="-- Choose a Tenant --"
                            disabled={tenants.length === 0 || tenantOptions.length === 0 || isRemovingTenant}
                            className="col-start-1 row-start-1 w-full"
                        />
   
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleDeleteTenant}
                            disabled={!selectedTenantId || isRemovingTenant}
                            className="col-start-2 row-start-1 text-red-600 border-red-300 hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed"
                            aria-label="Delete selected tenant"
                        >
                            {isRemovingTenant ? (
                                <span className="animate-spin inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full" role="status" aria-label="loading"></span>
                            ) : (
                                <MinusCircle className="w-4 h-4" />
                            )}
                        </Button>
   
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleAddNewTenant}
                            className="col-start-1 row-start-2 w-full flex items-center justify-center"
                            disabled={isLoadingTenants || isRemovingTenant}
                        >
                            <PlusCircle className="w-4 h-4 mr-2" /> Add New Tenant
                        </Button>
   
                        {tenants.length === 0 && !isLoadingTenants && (
                            <p className="col-start-1 col-span-2 text-xs text-gray-500">
                                No tenants available. Click 'Add New Tenant' to get started.
                            </p>
                        )}

                        {showAddTenantForm && (
                            <div className="col-start-1 row-start-3 w-full items-center justify-left">
                                <AddTenantForm
                                    onCancel={() => setShowAddTenantForm(false)}
                                    onSuccess={() => {
                                        setShowAddTenantForm(false);
                                    }}
                                />
                            </div>
                        )}
                    </div>
                )}
            </div>

            <h2 className="text-2xl font-semibold text-gray-700 mb-4">
                {selectedTenantId ? `Resources for ${selectedTenantName}` : 'Global Resources Overview'}
            </h2>

            {/* Dashboard Grid */}
            <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                {/* VPCs Card */}
                <Card>
                    <CardHeader className="flex justify-between items-center">
                        <CardTitle><Network className="w-5 h-5 mr-2 text-blue-500" />VPCs {isFetchingVpcs ? '(Refreshing)' : ''}</CardTitle>
                        <Button variant="outline" size="sm" onClick={() => setShowAddVpcForm(prev => !prev)} disabled={isRemovingVpc || !selectedTenantId}>
                            <PlusCircle className="w-4 h-4 mr-1" /> {showAddVpcForm ? 'Cancel' : 'Add VPC'}
                        </Button>
                    </CardHeader>
                    <CardContent>
                        {showAddVpcForm && selectedTenantId && <AddVpcForm onSuccess={() => setShowAddVpcForm(false)} tenantId={selectedTenantId} />}
                        {isLoadingVpcs ? <Skeleton className="h-20 w-full" /> : isErrorVpcs ? <ErrorMessage message={getErrorMessage(errorVpcs)} /> : 
                            <VpcTable vpcs={vpcs} onRemove={handleRemoveVpc} removingVpcVariables={removingVpcVariables?.name} />
                        }
                    </CardContent>
                </Card>

                {/* VMs Card */}
                <Card>
                    <CardHeader><CardTitle><Server className="w-5 h-5 mr-2 text-green-500" /> VMs {selectedTenantId ? `for ${selectedTenantName}` : ''} {isFetchingVms ? '(Refreshing)' : ''} </CardTitle></CardHeader>
                    <CardContent>
                        {!selectedTenantId ? <p className="text-sm text-gray-500 italic">Select tenant to view/manage VMs.</p>
                            : isLoadingVms ? <Skeleton className="h-24 w-full" />
                            : isErrorVms ? <ErrorMessage message={getErrorMessage(errorVms)} />
                            : <VMList vms={vms} tenantName={selectedTenantName} />
                        }
                        {selectedTenantId && (
                            <div className="mt-4 pt-4 border-t border-gray-200">

                                {!showCreateForm && (
                                    <Button onClick={() => setShowCreateForm(true)}>
                                        Create New VM
                                    </Button>
                                )}

                                {showCreateForm && (
                                    <CreateVMForm
                                        tenantId={selectedTenantId}
                                        tenantName={selectedTenantName}
                                        onClose={handleFormClose}
                                    />
                                )}
                            </div>
                         )}
                    </CardContent>
                </Card>

                {/* Provider Networks Card */}
                <Card>
                    <CardHeader><CardTitle><Globe className="w-5 h-5 mr-2 text-cyan-600" />Provider Networks {isFetchingProviderNets ? '(Refreshing)' : ''}</CardTitle></CardHeader>
                    <CardContent>
                        {isLoadingProviderNets ? <Skeleton className="h-20 w-full" /> : isErrorProviderNets ? <ErrorMessage message={getErrorMessage(errorProviderNets)} /> : <ProviderNetworkList networks={providerNetworks} />}
                    </CardContent>
                </Card>

                {/* SSH Keys Card */}
                <Card>
                    <CardHeader className="flex justify-between items-center">
                        <CardTitle><KeyRound className="w-5 h-5 mr-2 text-gray-600" />SSH Keys {isFetchingSSHKeys ? '(Refreshing)' : ''}</CardTitle>
                        <Button variant="outline" size="sm" onClick={() => setShowAddKeyForm(prev => !prev)} disabled={isRemovingSshKey || !selectedTenantId}>
                            <PlusCircle className="w-4 h-4 mr-1" /> {showAddKeyForm ? 'Cancel' : 'Add Key'}
                        </Button>
                    </CardHeader>
                    <CardContent>
                        {showAddKeyForm && selectedTenantId && <AddSSHKeyForm onSuccess={() => setShowAddKeyForm(false)} tenantId={selectedTenantId} />}
                        {isLoadingSSHKeys ? <Skeleton className="h-20 w-full mt-4" /> : isErrorSSHKeys ? <ErrorMessage message={getErrorMessage(errorSSHKeys)} /> :
                            <SSHKeyList sshKeys={sshKeys} onRemove={handleRemoveSshKey} removingKeyName={removingSSHVariables?.name}/>
                        }
                    </CardContent>
                </Card>

                {/* Hypervisors Card */}
                <Card>
                    <CardHeader><CardTitle><Globe className="w-5 h-5 mr-2 text-cyan-600" />Hypervisors {isFetchingHypervisors ? '(Refreshing)' : ''}</CardTitle></CardHeader>
                    <CardContent>
                        {isLoadingHypervisors ? <Skeleton className="h-20 w-full" /> : isErrorHypervisors ? <ErrorMessage message={getErrorMessage(errorHypervisors)} /> : <HypervisorList hypervisors={hypervisors} />}
                    </CardContent>
                </Card>
            </div>

            {!selectedTenantId && !isLoadingTenants && tenants.length > 0 && ( <div className="text-center text-gray-500 mt-10 flex flex-col items-center"> <Building size={48} className="mb-4 text-gray-400" /> <p>Select a tenant to view tenant-specific resources like VMs, VPCs or SSH keys.</p> </div> )}
        
        </div>
    );
}

export default Dashboard;
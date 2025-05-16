import { data } from "autoprefixer";
import axios from "axios";

const API_BASE = "http://192.168.1.15:8080";

export const listTenants = () => axios.get(`${API_BASE}/tenants/list`).then(res => res.data);
export const listProviderNetworks = () => axios.get(`${API_BASE}/provider_networks/list`).then(res => res.data);
export const listHypervisors = () => axios.get(`${API_BASE}/hypervisors/list`).then(res => res.data);

export const listObjects = (endpoint, tenantId) => {
    if (tenantId) {
        const payload = { id: tenantId };
        const url = `${API_BASE}/${endpoint}/list`;

        // console.log(`Requesting VPCs for tenant ${tenantId} via POST to ${url} with payload:`, payload);

        return axios.post(url, payload)
            .then(res => {
                return res.data;
            })
            .catch(error => {
                console.error(`Error fetching VPCs for tenant ${tenantId} from ${url}:`, error.response?.data || error.message);
                return [];
            });
    } else {
        console.log(`Tenant ID not provided. Not requesting VPCs. Returning empty list.`);
        return Promise.resolve([]);
    }
}

export const addSSHKey = (data) => {
    console.log("API: Adding SSH Key", data.name);
    return axios.post(`${API_BASE}/ssh_pub_key/create`, data).then(res => res.data);
};

export const removeSSHKey = (data) => {
    console.log("API: Removing SSH Key ID:", data.name);
    return axios.post(`${API_BASE}/ssh_pub_key/delete`, data).then(res => res.data);
};

export const createVM = (data) => {
    console.log("API: Creating VM", data);
    return axios.post(`${API_BASE}/virtualmachine/create`, data);
}
export const deleteVM = (data) => {
    console.log("API: Deleting VM", data);
    return axios.post(`${API_BASE}/virtualmachine/delete`, data);
}

export const addVpc = (data) => {
    console.log("API: Adding VPC", data);
    return axios.post(`${API_BASE}/vpc/create`, data);
}

export const removeVpc = (data) => {
    console.log("API: Deleting VPC", data);
    return axios.post(`${API_BASE}/vpc/delete`, data);
}

export const addTenant = (data) => {
    console.log("API: Adding Tenant", data);
    return axios.post(`${API_BASE}/tenant/create`, data);
}

export const removeTenant = (data) => {
    console.log("API: Removing Tenant", data);
    return axios.post(`${API_BASE}/tenant/delete`, data);
}

axios.interceptors.response.use(response => response, error => {
  console.error('API Error:', error.response || error.message);
  return Promise.reject(error);
});
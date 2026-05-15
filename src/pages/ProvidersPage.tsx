import { useEffect, useState } from 'react';
import {
  Title1,
  Button,
  Table,
  TableHeader,
  TableRow,
  TableHeaderCell,
  TableBody,
  TableCell,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Label,
  Input,
  Dropdown,
  Option,
  makeStyles,
  Spinner,
} from '@fluentui/react-components';
import { Add24Regular, Delete24Regular } from '@fluentui/react-icons';
import { useProviderStore } from '../stores/providerStore';
import type { ProviderInput } from '../types';

const useStyles = makeStyles({
  toolbar: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  formField: {
    marginBottom: '12px',
  },
});

const PROV_TYPES = ['openai', 'anthropic', 'openai_compatible'];

export default function ProvidersPage() {
  const styles = useStyles();
  const { providers, loading, error, fetchProviders, addProvider, deleteProvider } = useProviderStore();
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState<ProviderInput>({
    name: '',
    prov_type: 'openai',
    base_url: '',
    api_key: '',
    extra_headers: '',
  });

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const handleSubmit = async () => {
    await addProvider(form);
    setOpen(false);
    setForm({ name: '', prov_type: 'openai', base_url: '', api_key: '', extra_headers: '' });
  };

  return (
    <div>
      <div className={styles.toolbar}>
        <Title1>Providers</Title1>
        <Dialog open={open} onOpenChange={(_, data) => setOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button icon={<Add24Regular />}>Add Provider</Button>
          </DialogTrigger>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>Add Provider</DialogTitle>
              <DialogContent>
                <div className={styles.formField}>
                  <Label>Name</Label>
                  <Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
                </div>
                <div className={styles.formField}>
                  <Label>Type</Label>
                  <Dropdown value={form.prov_type} onOptionSelect={(_, data) => setForm({ ...form, prov_type: data.optionValue || 'openai' })}>
                    {PROV_TYPES.map((t) => (
                      <Option key={t} value={t}>{t}</Option>
                    ))}
                  </Dropdown>
                </div>
                <div className={styles.formField}>
                  <Label>Base URL</Label>
                  <Input value={form.base_url} onChange={(e) => setForm({ ...form, base_url: e.target.value })} placeholder="https://api.example.com/v1" />
                </div>
                <div className={styles.formField}>
                  <Label>API Key</Label>
                  <Input type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} />
                </div>
                <div className={styles.formField}>
                  <Label>Extra Headers (JSON)</Label>
                  <Input value={form.extra_headers || ''} onChange={(e) => setForm({ ...form, extra_headers: e.target.value })} placeholder='{"X-Custom": "value"}' />
                </div>
              </DialogContent>
              <DialogActions>
                <Button appearance="primary" onClick={handleSubmit}>Save</Button>
                <Button onClick={() => setOpen(false)}>Cancel</Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>
      </div>

      {loading && <Spinner />}
      {error && <div style={{ color: 'red' }}>{error}</div>}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHeaderCell>Name</TableHeaderCell>
            <TableHeaderCell>Type</TableHeaderCell>
            <TableHeaderCell>Base URL</TableHeaderCell>
            <TableHeaderCell>API Key</TableHeaderCell>
            <TableHeaderCell>Actions</TableHeaderCell>
          </TableRow>
        </TableHeader>
        <TableBody>
          {providers.map((p) => (
            <TableRow key={p.id}>
              <TableCell>{p.name}</TableCell>
              <TableCell>{p.prov_type}</TableCell>
              <TableCell>{p.base_url}</TableCell>
              <TableCell>{p.api_key}</TableCell>
              <TableCell>
                <Button icon={<Delete24Regular />} onClick={() => deleteProvider(p.id)} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

import { useEffect, useState } from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Table from '@mui/material/Table';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import TableBody from '@mui/material/TableBody';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';
import TextField from '@mui/material/TextField';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import CircularProgress from '@mui/material/CircularProgress';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import AddIcon from '@mui/icons-material/Add';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { useProviderStore } from '../stores/providerStore';
import { useI18n } from '../i18n';
import type { Provider, ProviderInput } from '../types';

const PROV_TYPES = ['openai', 'anthropic', 'openai_compatible'];

export default function ProvidersPage() {
  const { t } = useI18n();
  const { providers, loading, error, fetchProviders, addProvider, updateProvider, deleteProvider } = useProviderStore();
  const [open, setOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
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

  const resetForm = () => {
    setForm({ name: '', prov_type: 'openai', base_url: '', api_key: '', extra_headers: '' });
    setEditingId(null);
  };

  const handleOpenAdd = () => {
    resetForm();
    setOpen(true);
  };

  const handleOpenEdit = (p: Provider) => {
    setForm({
      name: p.name,
      prov_type: p.prov_type,
      base_url: p.base_url,
      api_key: p.api_key,
      extra_headers: p.extra_headers || '',
    });
    setEditingId(p.id);
    setOpen(true);
  };

  const handleSubmit = async () => {
    if (editingId) {
      await updateProvider(editingId, form);
    } else {
      await addProvider(form);
    }
    setOpen(false);
    resetForm();
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3, pb: 2, borderBottom: '1px solid', borderColor: 'divider' }}>
        <Typography variant="h4" sx={{ fontWeight: 700, letterSpacing: -0.5 }}>{t.providers.title}</Typography>
        <Button variant="contained" startIcon={<AddIcon />} onClick={handleOpenAdd}>
          {t.providers.addProvider}
        </Button>
      </Box>

      {loading && <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2, borderRadius: 2 }}>{t.common.error}: {error}</Alert>
      )}

      <Dialog open={open} onClose={() => { setOpen(false); resetForm(); }} maxWidth="sm" fullWidth slotProps={{ paper: { sx: { borderRadius: 3 } } }}>
        <DialogTitle sx={{ fontWeight: 700 }}>{editingId ? t.providers.editProvider : t.providers.addProvider}</DialogTitle>
        <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
          <TextField
            label={t.providers.name}
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            placeholder={t.providers.placeholderName}
            fullWidth
          />
          <FormControl fullWidth>
            <InputLabel>{t.providers.type}</InputLabel>
            <Select
              value={form.prov_type}
              label={t.providers.type}
              onChange={(e) => setForm({ ...form, prov_type: e.target.value })}
              MenuProps={{
                slotProps: {
                  paper: {
                    sx: {
                      mt: 0.5,
                      borderRadius: 2,
                      boxShadow: (theme: any) => theme.shadows[8],
                    },
                  },
                },
              }}
            >
              {PROV_TYPES.map((pt) => (
                <MenuItem key={pt} value={pt}>{pt}</MenuItem>
              ))}
            </Select>
          </FormControl>
          <TextField
            label={t.providers.baseUrl}
            value={form.base_url}
            onChange={(e) => setForm({ ...form, base_url: e.target.value })}
            placeholder={t.providers.placeholderUrl}
            fullWidth
          />
          <TextField
            label={t.providers.apiKey}
            type="password"
            value={form.api_key}
            onChange={(e) => setForm({ ...form, api_key: e.target.value })}
            fullWidth
          />
          <TextField
            label={t.providers.extraHeaders}
            value={form.extra_headers || ''}
            onChange={(e) => setForm({ ...form, extra_headers: e.target.value })}
            placeholder={t.providers.placeholderHeaders}
            fullWidth
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => { setOpen(false); resetForm(); }}>{t.providers.cancel}</Button>
          <Button variant="contained" onClick={handleSubmit}>{t.providers.save}</Button>
        </DialogActions>
      </Dialog>

      <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider', overflow: 'hidden' }}>
        <Table>
          <TableHead>
            <TableRow sx={{ bgcolor: 'action.hover' }}>
              <TableCell sx={{ fontWeight: 700 }}>{t.providers.name}</TableCell>
              <TableCell sx={{ fontWeight: 700 }}>{t.providers.type}</TableCell>
              <TableCell sx={{ fontWeight: 700 }}>{t.providers.baseUrl}</TableCell>
              <TableCell sx={{ fontWeight: 700 }}>{t.providers.apiKey}</TableCell>
              <TableCell align="right" sx={{ fontWeight: 700 }}>{t.providers.actions}</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {providers.map((p) => (
              <TableRow key={p.id} hover sx={{ '&:last-child td': { borderBottom: 0 } }}>
                <TableCell sx={{ fontWeight: 600 }}>{p.name}</TableCell>
                <TableCell>
                  <Chip size="small" variant="outlined" label={p.prov_type} />
                </TableCell>
                <TableCell sx={{ fontFamily: 'monospace', fontSize: '12px' }}>{p.base_url}</TableCell>
                <TableCell sx={{ fontFamily: 'monospace' }}>{p.api_key}</TableCell>
                <TableCell align="right">
                  <IconButton size="small" color="primary" onClick={() => handleOpenEdit(p)}>
                    <EditIcon fontSize="small" />
                  </IconButton>
                  <IconButton size="small" color="error" onClick={() => deleteProvider(p.id)}>
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </TableCell>
              </TableRow>
            ))}
            {providers.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} align="center" sx={{ py: 8, color: 'text.secondary' }}>
                  {t.dashboard.noData}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </Card>
    </Box>
  );
}

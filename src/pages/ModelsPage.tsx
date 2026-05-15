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
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import CircularProgress from '@mui/material/CircularProgress';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import { useModelStore } from '../stores/modelStore';
import { useProviderStore } from '../stores/providerStore';
import { useI18n } from '../i18n';
import type { ModelInput } from '../types';

export default function ModelsPage() {
  const { t } = useI18n();
  const { models, loading, error, fetchModels, addModel, deleteModel } = useModelStore();
  const { providers, fetchProviders } = useProviderStore();
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState<ModelInput>({
    provider_id: '',
    exposed_name: '',
    upstream_name: '',
    enabled: true,
    input_price: 0,
    output_price: 0,
  });

  useEffect(() => {
    fetchModels();
    fetchProviders();
  }, [fetchModels, fetchProviders]);

  const handleSubmit = async () => {
    await addModel(form);
    setOpen(false);
    setForm({ provider_id: '', exposed_name: '', upstream_name: '', enabled: true, input_price: 0, output_price: 0 });
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4" sx={{ fontWeight: 600 }}>{t.models.title}</Typography>
        <Button variant="contained" startIcon={<AddIcon />} onClick={() => setOpen(true)}>
          {t.models.addModel}
        </Button>
      </Box>

      {loading && <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>{t.common.error}: {error}</Alert>
      )}

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{t.models.addModel}</DialogTitle>
        <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
          <FormControl fullWidth>
            <InputLabel>{t.models.provider}</InputLabel>
            <Select
              value={form.provider_id}
              label={t.models.provider}
              onChange={(e) => setForm({ ...form, provider_id: e.target.value })}
            >
              {providers.map((p) => (
                <MenuItem key={p.id} value={p.id}>{p.name}</MenuItem>
              ))}
            </Select>
          </FormControl>
          <TextField
            label={t.models.exposedName}
            value={form.exposed_name}
            onChange={(e) => setForm({ ...form, exposed_name: e.target.value })}
            placeholder={t.models.placeholderExposed}
            fullWidth
          />
          <TextField
            label={t.models.upstreamName}
            value={form.upstream_name}
            onChange={(e) => setForm({ ...form, upstream_name: e.target.value })}
            placeholder={t.models.placeholderUpstream}
            fullWidth
          />
          <TextField
            label={t.models.inputPrice}
            type="number"
            value={form.input_price}
            onChange={(e) => setForm({ ...form, input_price: parseFloat(e.target.value) || 0 })}
            fullWidth
          />
          <TextField
            label={t.models.outputPrice}
            type="number"
            value={form.output_price}
            onChange={(e) => setForm({ ...form, output_price: parseFloat(e.target.value) || 0 })}
            fullWidth
          />
          <FormControlLabel
            control={
              <Switch
                checked={form.enabled}
                onChange={(e) => setForm({ ...form, enabled: e.target.checked })}
              />
            }
            label={t.models.enabled}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpen(false)}>{t.models.cancel}</Button>
          <Button variant="contained" onClick={handleSubmit}>{t.models.save}</Button>
        </DialogActions>
      </Dialog>

      <Card variant="outlined">
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>{t.models.exposedName}</TableCell>
              <TableCell>{t.models.upstreamName}</TableCell>
              <TableCell>{t.models.provider}</TableCell>
              <TableCell>{t.models.enabled}</TableCell>
              <TableCell>{t.models.inputPrice}</TableCell>
              <TableCell>{t.models.outputPrice}</TableCell>
              <TableCell align="right">{t.models.actions}</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {models.map((m) => (
              <TableRow key={m.id} hover>
                <TableCell sx={{ fontWeight: 600 }}>{m.exposed_name}</TableCell>
                <TableCell sx={{ fontFamily: 'monospace', fontSize: '12px' }}>{m.upstream_name}</TableCell>
                <TableCell>{providers.find((p) => p.id === m.provider_id)?.name || m.provider_id}</TableCell>
                <TableCell>
                  <Chip size="small" variant="outlined" color={m.enabled ? 'success' : 'default'} label={m.enabled ? 'Yes' : 'No'} />
                </TableCell>
                <TableCell>{m.input_price}</TableCell>
                <TableCell>{m.output_price}</TableCell>
                <TableCell align="right">
                  <IconButton size="small" color="error" onClick={() => deleteModel(m.id)}>
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </TableCell>
              </TableRow>
            ))}
            {models.length === 0 && (
              <TableRow>
                <TableCell colSpan={7} align="center" sx={{ py: 6, color: 'text.secondary' }}>
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
